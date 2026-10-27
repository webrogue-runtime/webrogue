use std::path::PathBuf;

use clap::Parser;
#[cfg(feature = "compile")]
mod android_gen_key;
#[cfg(feature = "hub")]
mod hub;
#[cfg(feature = "internal")]
mod internal;
mod licenses;
#[cfg(feature = "run")]
mod run;
mod schema_dump;

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
// #[command(propagate_version = true)]
enum Cli {
    /// Run a WRAPP file
    #[cfg(feature = "run")]
    Run {
        #[command(flatten)]
        command: run::RunCommand,
    },
    /// Builds native applications from WRAPP files
    #[cfg(feature = "compile")]
    Compile {
        // Path to cache config. See https://docs.wasmtime.dev/cli-cache.html
        #[arg(long)]
        cache: Option<PathBuf>,
        #[command(subcommand)]
        command: webrogue_aot_compiler::Commands,
    },
    Pack {
        /// Path to webrogue.json file
        #[arg(short, long)]
        config: PathBuf,
        /// Path to put resulting WRAPP file to
        /// If specified path is directory, then file named out.wrapp will be created in this directory
        #[arg(short, long)]
        output: PathBuf,
    },
    /// Commands related to Webrogue-Hub
    #[cfg(feature = "hub")]
    Hub {
        #[command(subcommand)]
        command: crate::hub::HubCommand,
    },
    /// Some commands that shouldn't be in release build
    #[cfg(feature = "internal")]
    Internal {
        #[command(subcommand)]
        command: crate::internal::InternalCommand,
    },
    /// Show licenses of Webrogue CLI and of it's dependencies and embedded resources
    Licenses {
        #[command(flatten)]
        command: crate::licenses::LicensesCommand,
    },
    /// An internal command to to dump JSON schemas
    SchemaDump {
        #[command(flatten)]
        command: crate::schema_dump::SchemaDumpCommand,
    },
    /// An internal command that runs the bundled LLD.
    /// The first argument selects the LLD driver (for example, ld.lld or lld-link).
    #[cfg(feature = "llvm")]
    #[command(hide = true, disable_help_flag = true)]
    Lld {
        #[arg(
            value_name = "ARG",
            trailing_var_arg = true,
            allow_hyphen_values = true,
            required = true
        )]
        args: Vec<String>,
    },
    // Some sort of bug in Clap prevents using command(flatten) here
    /// Generate keystore file usable for signing android applications.
    /// Note that you can do the same using "keytool" and "openssl" utilities
    #[cfg(feature = "compile")]
    AndroidGenKey {
        /// Path (usually with .p12 file extension) to put resulting PKCS12 keystore file to
        output: PathBuf,
        #[arg(long)]
        alias: String,
        #[arg(long)]
        password: String,
    },
    #[cfg(feature = "wasm-opt")]
    #[command(hide = true, disable_help_flag = true)]
    WasmOpt {
        #[arg(
            value_name = "ARG",
            trailing_var_arg = true,
            allow_hyphen_values = true,
            required = true
        )]
        args: Vec<String>,
    },
}

pub fn main() -> anyhow::Result<()> {
    let args = Cli::parse();
    #[cfg(debug_assertions)]
    {
        #[cfg(feature = "internal")]
        let is_internal = matches!(&args, Cli::Internal { .. });
        #[cfg(not(feature = "internal"))]
        let is_internal = false;

        // Keep host diagnostics out of the guest's captured stderr in tests.
        if !is_internal {
            tracing_subscriber::fmt().init();
        }
    }
    #[cfg(feature = "__rustls")]
    let _ = rustls::crypto::aws_lc_rs::default_provider().install_default();

    match args {
        #[cfg(feature = "run")]
        Cli::Run { command } => {
            command.run()?;
            Ok(())
        }
        #[cfg(feature = "compile")]
        Cli::Compile { command, cache } => {
            command.run(cache.as_ref())?;
            Ok(())
        }
        Cli::Pack { config, output } => {
            let vfs = webrogue_vfs::VFS::build_real(&config)?;
            webrogue_vfs::archive_to_file(
                &vfs,
                &output,
                webrogue_vfs::ArchiveOptions { keep_wasm: true },
            )?;
            Ok(())
        }
        #[cfg(feature = "hub")]
        Cli::Hub { command } => {
            command.run()?;
            Ok(())
        }
        #[cfg(feature = "internal")]
        Cli::Internal { command } => command.run(),
        Cli::Licenses { command } => {
            command.run()?;
            Ok(())
        }
        Cli::SchemaDump { command } => {
            command.run()?;
            Ok(())
        }
        #[cfg(feature = "llvm")]
        Cli::Lld { args } => {
            webrogue_lld::run_lld(args)?;
            Ok(())
        }
        #[cfg(feature = "compile")]
        Cli::AndroidGenKey {
            output,
            alias,
            password,
        } => {
            crate::android_gen_key::AndroidKeygen {
                output,
                alias,
                password,
            }
            .run()?;
            Ok(())
        }
        #[cfg(feature = "wasm-opt")]
        Cli::WasmOpt { args } => {
            let mut command = wasm_opt::integration::Command::new("whatever");
            command.args(args);
            wasm_opt::integration::run_from_command_args(command)?;
            Ok(())
        }
    }
}
