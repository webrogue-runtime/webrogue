pub fn build<
    FilePosition: webrogue_wrapp::IFilePosition,
    FileReader: webrogue_wrapp::IFileReader,
    VFSHandle: webrogue_wrapp::IVFSHandle<FilePosition, FileReader>,
    VFSBuilder: webrogue_wrapp::IVFSBuilder<FilePosition, FileReader, VFSHandle>,
>(
    build_dir: &std::path::PathBuf,
    vfs_builder: &mut VFSBuilder,
    old_stamp: Option<&webrogue_icons::IconsData>,
) -> anyhow::Result<webrogue_icons::IconsData> {
    let new_stamp = webrogue_icons::IconsData::from_vfs_builder(vfs_builder)?;
    if old_stamp != Some(&new_stamp) {
        webrogue_icons::android::generate_icons(build_dir, &new_stamp)?;
    }
    Ok(new_stamp)
}
