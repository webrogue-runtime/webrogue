pub fn build_apple_xcode(
    container_path: std::path::PathBuf,
    build_dir: std::path::PathBuf,
) -> anyhow::Result<()> {
    let template_dir = std::path::PathBuf::from("aot_artifacts/apple_xcode/template");

    crate::utils::copy_dir(&template_dir, &build_dir)?;

    let aot_dir = build_dir.join("aot");
    if !aot_dir.exists() {
        std::fs::create_dir(aot_dir.clone())?;
    }
    std::fs::copy(container_path.clone(), aot_dir.join("aot.wrapp"))?;

    let aot_dir = build_dir.join("aot");

    crate::compile::compile_wrapp_to_object(
        container_path.clone(),
        aot_dir.join("aot.x86_64.macosx.o"),
        crate::Target::x86_64AppleDarwin,
        true, // TODO check
    )?;
    crate::compile::compile_wrapp_to_object(
        container_path.clone(),
        aot_dir.join("aot.arm64.macosx.o"),
        crate::Target::ARM64AppleDarwin,
        true, // TODO check
    )?;
    crate::compile::compile_wrapp_to_object(
        container_path.clone(),
        aot_dir.join("aot.x86_64.iphonesimulator.o"),
        crate::Target::X86_64AppleIOSSIM,
        true, // TODO check
    )?;
    crate::compile::compile_wrapp_to_object(
        container_path.clone(),
        aot_dir.join("aot.arm64.iphonesimulator.o"),
        crate::Target::ARM64AppleIOSSIM,
        true, // TODO check
    )?;
    crate::compile::compile_wrapp_to_object(
        container_path.clone(),
        aot_dir.join("aot.arm64.iphoneos.o"),
        crate::Target::ARM64AppleIOS,
        true, // TODO check
    )?;

    return anyhow::Ok(());
}
