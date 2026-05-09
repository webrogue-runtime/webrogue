use webrogue_cli_goodies::step;

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
            step("Compiling AOT object for x86_64 macOS".to_owned(), || {
                crate::compile::compile_wrapp_to_object(
                    wrapp_path,
                    &aot_dir.join("aot.x86_64.macosx.o"),
                    crate::Target::x86_64AppleDarwin,
                    cache,
                    true, // TODO check
                    false,
                )
            })?;
            step("Compiling AOT object for AArch64 macOS".to_owned(), || {
                crate::compile::compile_wrapp_to_object(
                    wrapp_path,
                    &aot_dir.join("aot.arm64.macosx.o"),
                    crate::Target::ARM64AppleDarwin,
                    cache,
                    true, // TODO check
                    false,
                )
            })?;
        }
        Destination::Ios => {
            step("Compiling AOT object for AArch64 iOS".to_owned(), || {
                crate::compile::compile_wrapp_to_object(
                    wrapp_path,
                    &aot_dir.join("aot.arm64.iphoneos.o"),
                    crate::Target::ARM64AppleIOS,
                    cache,
                    true, // TODO check
                    false,
                )
            })?;
        }
        Destination::IOSSim => {
            step(
                "Compiling AOT object for AArch64 iOS simulator".to_owned(),
                || {
                    crate::compile::compile_wrapp_to_object(
                        wrapp_path,
                        &aot_dir.join("aot.arm64.iphonesimulator.o"),
                        crate::Target::ARM64AppleIOSSIM,
                        cache,
                        true, // TODO check
                        false,
                    )
                },
            )?;
            step(
                "Compiling AOT object for x86_64 iOS simulator".to_owned(),
                || {
                    crate::compile::compile_wrapp_to_object(
                        wrapp_path,
                        &aot_dir.join("aot.x86_64.iphonesimulator.o"),
                        crate::Target::X86_64AppleIOSSIM,
                        cache,
                        true, // TODO check
                        false,
                    )
                },
            )?;
        }
    }
    Ok(())
}
