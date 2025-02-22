pub fn copy_dir(source: &std::path::PathBuf, dest: &std::path::PathBuf) -> anyhow::Result<()> {
    let mut path_parts = vec![];
    copy_dir_impl(source, dest, &mut path_parts)
}

fn copy_dir_impl(
    source: &std::path::PathBuf,
    dest: &std::path::PathBuf,
    parts: &mut Vec<String>,
) -> anyhow::Result<()> {
    let mut source_path = source.clone();
    let mut dest_path = dest.clone();
    for part in parts.clone() {
        source_path.push(part.clone());
        dest_path.push(part.clone());
    }
    if !std::fs::exists(dest_path.clone())? {
        std::fs::create_dir(dest_path.clone())?;
    }
    for dir_entry in std::fs::read_dir(source_path.clone())? {
        let dir_entry = dir_entry?;
        let file_type = dir_entry.file_type()?;
        let name = dir_entry.file_name();
        if file_type.is_dir() {
            parts.push(name.clone().into_string().unwrap());
            copy_dir_impl(source, dest, parts)?;
            parts.pop().unwrap();
        } else if file_type.is_file() {
            let dest_file_path = dest_path.join(name.clone());
            let should_copy = if std::fs::exists(dest_file_path.clone())? {
                let source_modification_time = dir_entry.metadata()?.modified()?;
                let dest_modification_time =
                    std::fs::metadata(dest_file_path.clone())?.modified()?;
                source_modification_time > dest_modification_time
            } else {
                true
            };

            if should_copy {
                std::fs::copy(source_path.join(name.clone()), dest_file_path)?;
            }
        }
    }

    return anyhow::Ok(());
}

pub fn run_lld(_args: Vec<String>) -> anyhow::Result<()> {
    #[cfg(feature = "llvm")]
    return webrogue_aot_linker::run_lld(_args);
    #[cfg(not(feature = "llvm"))]
    anyhow::bail!("LLVM feature is disabled at build time")
}