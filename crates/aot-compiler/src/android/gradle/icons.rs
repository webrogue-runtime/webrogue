use webrogue_cli_goodies::step;
use webrogue_vfs::VFS;

pub fn build(
    build_dir: &std::path::Path,
    vfs: &VFS,
    old_stamp: Option<&webrogue_icons::IconsData>,
) -> anyhow::Result<webrogue_icons::IconsData> {
    let new_stamp = webrogue_icons::IconsData::from_vfs(vfs)?;
    if old_stamp != Some(&new_stamp) {
        step("Generating icons".to_owned(), || {
            webrogue_icons::android::generate_icons(build_dir, &new_stamp)
        })?;
    }
    Ok(new_stamp)
}
