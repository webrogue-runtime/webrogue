use std::path::{Path, PathBuf};

pub fn aot_persistent_dir_path(data_dir: Option<&Path>, wrapp_id: &str) -> PathBuf {
    data_dir
        .map(|data_dir| data_dir.to_path_buf())
        .unwrap_or_else(|| dirs::data_dir().expect("dirs::data_dir must return Some"))
        .join(wrapp_id)
        .join("persistent")
}

/// base_dir can be obtained from [cli_dir]
pub fn wrapp_persistent_dir_path(base_dir: &Path, wrapp_id: &str) -> PathBuf {
    base_dir.join(wrapp_id).join("persistent")
}

#[cfg(not(windows))]
const CLI_DIR_NAME: &str = "webrogue";
#[cfg(windows)]
const CLI_DIR_NAME: &str = "Webrogue";

/// Panics if dirs::data_local_dir or dirs::data_dir fails
pub fn cli_dir(is_local: bool) -> PathBuf {
    let data_dir = if is_local {
        dirs::data_local_dir().expect("dirs::data_local_dir must return Some")
    } else {
        dirs::data_dir().expect("dirs::data_dir must return Some")
    };
    data_dir.join(CLI_DIR_NAME)
}
