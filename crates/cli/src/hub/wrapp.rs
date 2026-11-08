use std::{io::SeekFrom, path::PathBuf};

use anyhow::Context as _;
use clap::Subcommand;
use tempfile::NamedTempFile;
use tokio::io::{AsyncReadExt as _, AsyncSeekExt as _};
use webrogue_hub_client::{
    api_base_path::http_api_url,
    openapi::{
        apis::{configuration::Configuration, default_api},
        models,
    },
};
use webrogue_vfs::ArchiveOptions;

const PART_BATCH: usize = 50;

#[derive(Subcommand, Debug, Clone)]
pub enum WrappCommand {
    /// Upload a WRAPP to Webrogue Hub
    Upload {
        /// Path to the WRAPP file
        wrapp_path: PathBuf,
        /// Name shown in Webrogue Hub (defaults to the file stem)
        #[arg(long)]
        name: Option<String>,
        /// Replace an existing WRAPP by ID instead of creating a new one. The
        /// WRAPP ID and its public URL are preserved.
        #[arg(long, conflicts_with = "name")]
        id: Option<String>,
    },
    /// Download a WRAPP from Webrogue Hub
    Download {
        /// WRAPP ID
        id: String,
        /// Output path (defaults to <name>.wrapp)
        #[arg(short, long)]
        output: Option<PathBuf>,
    },
    /// List WRAPPs owned by the current user
    List,
    /// Delete a WRAPP from Webrogue Hub
    Delete {
        /// WRAPP ID
        id: String,
    },
}

pub async fn run(command: &WrappCommand, api_key: &str) -> anyhow::Result<()> {
    let configuration = configuration(api_key);
    match command {
        WrappCommand::Upload {
            wrapp_path,
            name,
            id,
        } => {
            let wrapp_path = wrapp_path.clone();
            let packed_wrapp_path =
                webrogue_cli_goodies::step_async("Packing WRAPP file".to_string(), async move {
                    tokio::task::spawn_blocking(move || -> anyhow::Result<NamedTempFile> {
                        let vfs = webrogue_vfs::VFS::build_from_path(&wrapp_path)?;
                        let mut packed_wrapp_path = NamedTempFile::new_in(
                            wrapp_path
                                .parent()
                                .ok_or_else(|| anyhow::anyhow!("Path error"))?,
                        )?;
                        webrogue_vfs::archive(
                            &vfs,
                            &mut packed_wrapp_path,
                            ArchiveOptions { keep_wasm: true },
                        )?;
                        Ok(packed_wrapp_path)
                    })
                    .await?
                })
                .await?;
            upload(
                &configuration,
                packed_wrapp_path.path(),
                name.as_deref(),
                id.as_deref(),
            )
            .await?;
            drop(packed_wrapp_path);
            Ok(())
        }
        WrappCommand::Download { id, output } => {
            download(&configuration, id, output.as_deref()).await
        }
        WrappCommand::List => list(&configuration).await,
        WrappCommand::Delete { id } => delete(&configuration, id).await,
    }
}

fn configuration(api_key: &str) -> Configuration {
    let mut configuration = Configuration::new();
    configuration.bearer_access_token = Some(api_key.to_owned());
    configuration.base_path = http_api_url();
    configuration
}

async fn upload(
    configuration: &Configuration,
    wrapp_path: &std::path::Path,
    name: Option<&str>,
    replace_id: Option<&str>,
) -> anyhow::Result<()> {
    let (mut file, size) =
        webrogue_cli_goodies::step_async("Opening WRAPP file".to_string(), async {
            let file = tokio::fs::File::open(wrapp_path)
                .await
                .with_context(|| format!("Unable to open {}", wrapp_path.display()))?;
            let size = file
                .metadata()
                .await
                .with_context(|| format!("Unable to stat {}", wrapp_path.display()))?
                .len() as i64;
            anyhow::ensure!(size > 0, "{} is empty", wrapp_path.display());
            Ok::<_, anyhow::Error>((file, size))
        })
        .await?;

    let created = if let Some(id) = replace_id {
        let _ = default_api::abort_wrapp_upload(configuration, &id).await;
        webrogue_cli_goodies::step_async("Starting WRAPP replacement".to_string(), async {
            default_api::replace_wrapp(configuration, id, models::ReplaceWrappRequest::new(size))
                .await
                .with_context(|| format!("Unable to start replacing the WRAPP {id}"))
        })
        .await?
    } else {
        let name = name.map(str::to_owned).unwrap_or_else(|| {
            wrapp_path
                .file_stem()
                .and_then(|stem| stem.to_str())
                .unwrap_or("wrapp")
                .to_owned()
        });

        webrogue_cli_goodies::step_async("Creating WRAPP".to_string(), async {
            default_api::create_wrapp(configuration, models::CreateWrappRequest::new(name, size))
                .await
                .context("Unable to create the WRAPP")
        })
        .await?
    };
    let id = created.id.to_string();

    let result = webrogue_cli_goodies::step_async("Uploading WRAPP".to_string(), async {
        upload_parts(configuration, &id, &created, &mut file, size).await
    })
    .await;
    let wrapp = match result {
        Ok(wrapp) => wrapp,
        Err(error) => {
            // Do not leave a dangling multipart upload behind.
            let _ = default_api::abort_wrapp_upload(configuration, &id).await;
            return Err(error);
        }
    };

    println!("Uploaded (id: {}, name: {})", wrapp.id, wrapp.name);
    Ok(())
}

async fn upload_parts(
    configuration: &Configuration,
    id: &str,
    created: &models::CreateWrappResponse,
    file: &mut tokio::fs::File,
    size: i64,
) -> anyhow::Result<models::WrappInfo> {
    let part_numbers: Vec<i32> = (1..=created.part_count).collect();
    let mut completed = Vec::with_capacity(part_numbers.len());

    for batch in part_numbers.chunks(PART_BATCH) {
        let response = default_api::get_wrapp_part_urls(
            configuration,
            id,
            models::PartUrlsRequest::new(batch.to_vec()),
        )
        .await
        .context("Unable to get presigned upload URLs")?;

        for part in response.urls {
            let start = (part.part_number as i64 - 1) * created.part_size;
            let end = (start + created.part_size).min(size);
            anyhow::ensure!(
                start >= 0 && start <= end && end <= size,
                "Invalid byte range for part {}",
                part.part_number
            );
            let mut body = vec![0u8; (end - start) as usize];

            file.seek(SeekFrom::Start(start as u64))
                .await
                .with_context(|| format!("Unable to seek to part {}", part.part_number))?;
            file.read_exact(&mut body)
                .await
                .with_context(|| format!("Unable to read part {}", part.part_number))?;

            let response = configuration
                .client
                .put(&part.url)
                .body(body)
                .send()
                .await
                .with_context(|| format!("Unable to upload part {}", part.part_number))?;
            anyhow::ensure!(
                response.status().is_success(),
                "Upload of part {} failed: {}",
                part.part_number,
                response.status()
            );

            let etag = response
                .headers()
                .get("etag")
                .and_then(|value| value.to_str().ok())
                .map(str::to_owned)
                .with_context(|| format!("Part {} did not return an ETag", part.part_number))?;

            completed.push(models::CompletedPart::new(part.part_number, etag));
        }
    }

    completed.sort_by_key(|part| part.part_number);
    let response = default_api::complete_wrapp_upload(
        configuration,
        id,
        models::CompleteUploadRequest::new(completed),
    )
    .await
    .context("Unable to complete the upload")?;

    Ok(*response.wrapp)
}

async fn download(
    configuration: &Configuration,
    id: &str,
    output: Option<&std::path::Path>,
) -> anyhow::Result<()> {
    let wrapp = *default_api::get_wrapp(configuration, id)
        .await
        .context("Unable to get the WRAPP")?
        .wrapp;

    let url = match &wrapp.download_url {
        Some(url) => url.clone(),
        None => {
            default_api::get_wrapp_download_url(configuration, id)
                .await
                .context("Unable to get a download URL")?
                .url
        }
    };

    let response = configuration
        .client
        .get(url)
        .send()
        .await
        .context("Unable to download the WRAPP")?;
    anyhow::ensure!(
        response.status().is_success(),
        "Download failed: {}",
        response.status()
    );
    let data = response
        .bytes()
        .await
        .context("Unable to read the downloaded WRAPP")?;

    let output = output
        .map(std::path::Path::to_path_buf)
        .unwrap_or_else(|| default_output_path(&wrapp.name));
    std::fs::write(&output, &data)
        .with_context(|| format!("Unable to write {}", output.display()))?;

    println!("Downloaded {} to {}", wrapp.name, output.display());
    Ok(())
}

fn default_output_path(name: &str) -> PathBuf {
    if name.to_lowercase().ends_with(".wrapp") {
        PathBuf::from(name)
    } else {
        PathBuf::from(format!("{name}.wrapp"))
    }
}

async fn list(configuration: &Configuration) -> anyhow::Result<()> {
    let response = default_api::list_wrapps(configuration)
        .await
        .context("Unable to list WRAPPs")?;

    if response.wrapps.is_empty() {
        println!("No WRAPPs uploaded yet");
        return Ok(());
    }

    for wrapp in response.wrapps {
        let mut flags = Vec::new();
        if wrapp.is_public {
            flags.push("public");
        }
        if wrapp.is_example {
            flags.push("example");
        }
        let flags = if flags.is_empty() {
            String::new()
        } else {
            format!(" [{}]", flags.join(", "))
        };
        println!(
            "{}\t{}\t{} bytes{}",
            wrapp.id, wrapp.name, wrapp.size, flags
        );
    }
    Ok(())
}

async fn delete(configuration: &Configuration, id: &str) -> anyhow::Result<()> {
    default_api::delete_wrapp(configuration, id)
        .await
        .context("Unable to delete the WRAPP")?;
    println!("Deleted {id}");
    Ok(())
}
