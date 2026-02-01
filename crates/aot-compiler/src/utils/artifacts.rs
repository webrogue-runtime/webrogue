use anyhow::Context;

pub struct Artifacts {
    inner: Box<dyn ArtifactsInner>,
}

impl Artifacts {
    pub fn new() -> anyhow::Result<Artifacts> {
        #[cfg(feature = "appended_artifacts")]
        let archive = {
            use std::io::{Read as _, Seek as _};

            let mut current_file = std::fs::File::open(std::env::current_exe()?)?;
            let file_size = current_file.seek(std::io::SeekFrom::End(0))?;
            let mut wrapp_size_bytes = [0u8; 8];
            current_file.seek(std::io::SeekFrom::End(-8))?;
            current_file.read_exact(&mut wrapp_size_bytes)?;
            let wrapp_size = u64::from_le_bytes(wrapp_size_bytes);

            let reader = webrogue_wrapp::RangeReader::new(
                current_file,
                file_size - wrapp_size - 8,
                wrapp_size,
            )?;
            zip::ZipArchive::new(reader)?
        };
        #[cfg(not(feature = "appended_artifacts"))]
        let archive = {
            let path = std::env::var("WEBROGUE_ARTIFACTS_PATH")
                .ok()
                .unwrap_or_else(|| "aot_artifacts.zip".to_string());

            zip::ZipArchive::new(std::fs::File::open(path)?)?
        };
        Ok(Artifacts {
            inner: Box::new(ArtifactsImpl { archive }),
        })
    }

    pub fn extract_tmp<P: AsRef<std::path::Path>>(
        &mut self,
        base_path: P,
        file: &str,
    ) -> anyhow::Result<super::TemporalFile> {
        let result = super::TemporalFile::for_tmp(base_path.as_ref(), file.replace("/", "_"))?;
        self.inner
            .extract(result.path(), file)
            .with_context(|| format!("Unable to extract {} from archive", file))?;
        Ok(result)
    }

    pub fn extract<P: AsRef<std::path::Path>>(
        &mut self,
        out_path: P,
        file: &str,
    ) -> anyhow::Result<()> {
        self.inner
            .extract(out_path.as_ref(), file)
            .with_context(|| format!("Unable to extract {} from archive", file))?;
        Ok(())
    }

    // pub fn extract<P: AsRef<std::path::Path>>(
    //     &mut self,
    //     output_path: P,
    //     file: &str,
    // ) -> anyhow::Result<()> {
    //     self.inner.extract(output_path.as_ref(), file)
    // }

    pub fn get_data(&mut self, file: &str) -> anyhow::Result<Vec<u8>> {
        self.inner.get_data(file)
    }

    pub fn extract_dir<P: AsRef<std::path::Path>>(
        &mut self,
        output_path: P,
        dir: &str,
    ) -> anyhow::Result<()> {
        self.inner.extract_dir(output_path.as_ref(), dir)
    }
}

trait ArtifactsInner {
    fn extract(&mut self, output_path: &std::path::Path, file: &str) -> anyhow::Result<()>;
    fn get_data(&mut self, file: &str) -> anyhow::Result<Vec<u8>>;
    fn extract_dir(&mut self, output_path: &std::path::Path, dir: &str) -> anyhow::Result<()>;
}

struct ArtifactsImpl<R: std::io::Read + std::io::Seek> {
    archive: zip::ZipArchive<R>,
}

impl<R: std::io::Read + std::io::Seek> ArtifactsInner for ArtifactsImpl<R> {
    fn extract(&mut self, output_path: &std::path::Path, file: &str) -> anyhow::Result<()> {
        let mut zip_file = self.archive.by_name(file)?;
        let mut output_file = std::fs::File::create(output_path)?;
        std::io::copy(&mut zip_file, &mut output_file)?;
        Ok(())
    }

    fn get_data(&mut self, file: &str) -> anyhow::Result<Vec<u8>> {
        let mut zip_file = self.archive.by_name(file)?;
        let mut result = Vec::new();
        std::io::copy(&mut zip_file, &mut result)?;
        Ok(result)
    }

    fn extract_dir(&mut self, output_path: &std::path::Path, dir: &str) -> anyhow::Result<()> {
        let mut dir = dir.to_owned();
        if !dir.ends_with("/") {
            dir += "/";
        }
        let filenames = self
            .archive
            .file_names()
            .map(|filename| filename.to_owned())
            .collect::<Vec<_>>();
        for path in filenames {
            if !path.starts_with(&dir) {
                continue;
            }
            let relative_path = &path[dir.len()..];
            if self.archive.by_name(&path)?.is_dir() {
                continue;
            }
            let mut file_output_path = output_path.to_path_buf();
            for part in relative_path.split("/") {
                file_output_path = file_output_path.join(part);
            }

            let _ = std::fs::create_dir_all(file_output_path.parent().unwrap());

            self.extract(&file_output_path, &path)?;
        }
        Ok(())
    }
}
