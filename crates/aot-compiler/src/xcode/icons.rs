pub fn build(
    build_dir: &std::path::Path,
    wrapp_builder: &mut impl webrogue_wrapp::IVFSBuilder,
    old_stamp: Option<&webrogue_icons::IconsData>,
) -> anyhow::Result<webrogue_icons::IconsData> {
    let new_stamp = webrogue_icons::IconsData::from_vfs_builder(wrapp_builder)?;
    if old_stamp != Some(&new_stamp) {
        webrogue_icons::xcode::generate_icons(build_dir, &new_stamp)?;
    }
    Ok(new_stamp)
}
