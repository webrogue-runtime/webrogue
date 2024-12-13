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
    std::fs::copy(container_path.clone(), aot_dir.join("aot.webc"))?;

    let aot_dir = build_dir.join("aot");

    crate::compile::compile_webc_to_object(
        container_path.clone(),
        aot_dir.join("aot_x86_64.o"),
        "x86_64-apple-darwin",
    )?;
    crate::compile::compile_webc_to_object(
        container_path.clone(),
        aot_dir.join("aot_arm64.o"),
        "arm64-apple-darwin",
    )?;

    return anyhow::Ok(());
}
