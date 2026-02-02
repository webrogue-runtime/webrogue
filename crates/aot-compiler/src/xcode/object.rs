use super::types::Destination;

pub fn compile(
    destination: Destination,
    wrapp_path: &std::path::Path,
    build_dir: &std::path::Path,
    cache: Option<&std::path::PathBuf>,
) -> anyhow::Result<()> {
    let aot_dir = build_dir.join("aot");
    match destination {
        Destination::MacOS => {
            println!("Compiling AOT object for x86_64 macOS...");
            crate::compile::compile_wrapp_to_object(
                wrapp_path,
                &aot_dir.join("aot.x86_64.macosx.o"),
                crate::Target::x86_64AppleDarwin,
                cache,
                true, // TODO check
                false,
            )?;
            println!("Compiling AOT object for AArch64 macOS...");
            crate::compile::compile_wrapp_to_object(
                wrapp_path,
                &aot_dir.join("aot.arm64.macosx.o"),
                crate::Target::ARM64AppleDarwin,
                cache,
                true, // TODO check
                false,
            )?;
        }
        Destination::Ios => {
            println!("Compiling AOT object for AArch64 iOS...");
            crate::compile::compile_wrapp_to_object(
                wrapp_path,
                &aot_dir.join("aot.arm64.iphoneos.o"),
                crate::Target::ARM64AppleIOS,
                cache,
                true, // TODO check
                false,
            )?;
        }
        Destination::IOSSim => {
            println!("Compiling AOT object for AArch64 iOS simulator...");
            crate::compile::compile_wrapp_to_object(
                wrapp_path,
                &aot_dir.join("aot.arm64.iphonesimulator.o"),
                crate::Target::ARM64AppleIOSSIM,
                cache,
                true, // TODO check
                false,
            )?;
            println!("Compiling AOT object for x86_64 iOS simulator...");
            crate::compile::compile_wrapp_to_object(
                wrapp_path,
                &aot_dir.join("aot.x86_64.iphonesimulator.o"),
                crate::Target::X86_64AppleIOSSIM,
                cache,
                true, // TODO check
                false,
            )?;
        }
    }
    Ok(())
}
