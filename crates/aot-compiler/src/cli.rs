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
        /// CPU architecture to compile for.
        /// Defaults to x86_64.
        #[arg(long)]
        arch: Option<crate::linux::LinuxArch>,
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
        /// CPU architecture to compile for.
        /// Defaults to x86_64.
        #[arg(long)]
        arch: Option<crate::windows::WindowsArch>,
        /// Use console app's entry point.
        /// It allow stdin/stdout/stderr to work, but opens console window upon launch.
        /// It also makes cmd pop up on launch, so this option is not recommended.
        #[arg(long)]
        console: bool,
        /// Specify the Vulkan fallback renderer to use.
        /// Some Windows machines may lack support of Vulkan due to missing or outdated drivers or unsupported hardware.
        /// Webrogue provides two software Vulkan implementations as a fallback: SwiftShader and Lavapipe.
        /// Lavapipe has higher performance, but it's size is approximately 50 MB.
        /// Wberogue's build of SwiftShader uses "Subzero reactor" as code generator, which has slightly lower performance and is only available on x64, but it's size is only 5 MB.
        /// Webrogue places the selected Vulkan fallback renderer as .dll file in the same directory as the resulting executable.
        /// If no Vulkan fallback renderer is specified, the resulting executable will fail to start if hardware-accelerated rendering is required but unavailable.
        #[arg(long, value_name = "RENDERER")]
        vulkan_fallback: Option<crate::windows::VulkanFallback>,
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
                let object_file = crate::utils::TemporaryFile::for_tmp_object(out_path)?;
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
                arch,
            } => {
                crate::linux::build_linux(
                    wrapp_path,
                    out_path,
                    libc.clone().unwrap_or(crate::linux::LibC::GLibC),
                    arch.clone().unwrap_or(crate::linux::LinuxArch::X86_64),
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
                output,
                cache,
            )?,
            Commands::Windows {
                wrapp_path,
                out_path,
                arch,
                console,
                vulkan_fallback,
            } => crate::windows::build(
                wrapp_path,
                out_path,
                arch.as_ref()
                    .unwrap_or(&crate::windows::WindowsArch::X86_64)
                    .clone(),
                *console,
                cache,
                vulkan_fallback.clone(),
            )?,
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
