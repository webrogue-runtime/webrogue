use clap::Subcommand;

#[derive(Subcommand, Debug, Clone)]
pub enum Commands {
    /// Build Linux executable
    Linux {
        /// Path to WRAPP file
        wrapp_path: std::path::PathBuf,
        /// Path where resulting executable will be placed
        out_path: std::path::PathBuf,
        /// LibC to compile for.
        /// Defaults to glibc.
        #[arg(long)]
        libc: Option<crate::linux::LibC>,
    },
    /// Build Android app using Gradle project
    Android {
        /// Path to WRAPP file
        wrapp_path: std::path::PathBuf,
        /// Path where resulting project will be placed
        build_dir: std::path::PathBuf,
        /// Path to Android SDK.
        /// If not specified, ANDROID_SDK_ROOT or ANDROID_HOME environment variable will be used.
        #[arg(long, value_name = "PATH")]
        sdk: Option<std::path::PathBuf>,
        /// Path to Java installation directory.
        /// If not specified, JAVA_HOME environment variable is used.
        #[arg(long, value_name = "PATH")]
        java_home: Option<std::path::PathBuf>,
        /// Path to release signing keystore.
        ///
        /// Hint: keystore can be generated using following command:
        ///     keytool -genkeypair -keyalg RSA -keystore <PATH>.jks -alias <ALIAS> -validity 3650
        #[arg(long, value_name = "PATH")]
        keystore_path: Option<std::path::PathBuf>,
        /// Release store password. Usually same as key password.
        #[arg(long, value_name = "PASSWORD")]
        store_password: Option<String>,
        /// Release key password. Usually same as store password.
        #[arg(long, value_name = "PASSWORD")]
        key_password: Option<String>,
        /// Release key password
        #[arg(long, value_name = "ALIAS")]
        key_alias: Option<String>,
        /// Debug build. Applies only to Java code. Resulting APK will be slightly larger.
        #[arg(long)]
        debug: bool,
        /// Path to place resulting APK
        #[arg(short, long, value_name = "PATH")]
        output: Option<std::path::PathBuf>,
    },
    /// Build Windows app
    Windows {
        /// Path to WRAPP file
        wrapp_path: std::path::PathBuf,
        /// Path where resulting executable will be placed
        out_path: std::path::PathBuf,
        /// Use console app's entry point.
        /// It allow stdin/stdout/stderr to work, but opens console window upon launch.
        /// It also makes cmd pop up on launch, so this option is not recommended.
        #[arg(long)]
        console: bool,
        /// Don't add vk_swiftshader.dll.
        /// SwiftShader is used as a fallback renderer on system that have no Vulkan drivers installed.
        /// Webrogue places SwiftShader in the same directory resulting executable is in.
        /// It's recommended to keep SwiftShader in most cases, but you can use this option to skip this 
        /// step, so you app will fail to start if hardware-accelerated rendering is unavailable.
        #[arg(long)]
        no_swiftshader: bool,
    },
    /// Xcode-related commands
    Xcode {
        /// Path to WRAPP file
        wrapp_path: std::path::PathBuf,
        /// Path where resulting project will be placed
        build_dir: std::path::PathBuf,
        #[command(subcommand)]
        commands: crate::xcode::XcodeCommands,
    },
    /// An internal command.
    /// You probably need `android` command instead
    AndroidSo {
        wrapp_path: std::path::PathBuf,
        out_path: std::path::PathBuf,
        target: String,
    },
    /// An internal command
    Object {
        wrapp_path: std::path::PathBuf,
        out_path: std::path::PathBuf,
        target: String,
        #[arg(short, long)]
        pic: bool,
    },
}

impl Commands {
    pub fn run(&self, cache: Option<&std::path::PathBuf>) -> anyhow::Result<()> {
        match self {
            Commands::Object {
                wrapp_path,
                out_path,
                target,
                pic,
            } => {
                crate::compile::compile_wrapp_to_object(
                    wrapp_path,
                    out_path,
                    crate::Target::from_name(target)?,
                    cache,
                    *pic,
                    false,
                )?;
            }
            Commands::AndroidSo {
                wrapp_path,
                out_path,
                target,
            } => {
                let target = crate::Target::from_name(target)?;
                let object_file = crate::utils::TemporalFile::for_tmp_object(out_path)?;
                crate::compile::compile_wrapp_to_object(
                    wrapp_path,
                    object_file.path(),
                    target,
                    cache,
                    true,
                    true,
                )?;
                crate::android::link(&object_file, target, out_path)?;
            }
            Commands::Linux {
                wrapp_path,
                out_path,
                libc,
            } => {
                crate::linux::build_linux(
                    wrapp_path,
                    out_path,
                    libc.clone().unwrap_or(crate::linux::LibC::GLibC),
                    cache,
                )?;
            }
            Commands::Android {
                wrapp_path,
                build_dir,
                sdk,
                java_home,
                keystore_path,
                store_password,
                key_password,
                key_alias,
                debug,
                output,
            } => crate::android::build(
                wrapp_path,
                build_dir,
                sdk,
                java_home,
                keystore_path,
                store_password,
                key_password,
                key_alias,
                *debug,
                output,
                cache,
            )?,
            Commands::Windows {
                wrapp_path,
                out_path,
                console,
                no_swiftshader,
            } => crate::windows::build(wrapp_path, out_path, *console, cache, !no_swiftshader)?,
            Commands::Xcode {
                wrapp_path,
                build_dir,
                commands,
            } => crate::xcode::run(
                crate::xcode::XcodeArgs {
                    wrapp_path,
                    build_dir,
                    cache,
                },
                commands,
            )?,
        }

        Ok(())
    }
}
