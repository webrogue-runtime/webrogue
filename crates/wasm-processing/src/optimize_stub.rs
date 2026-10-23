use std::path::Path;

pub fn optimize(_component: &[u8], _tmp1: &Path, _tmp2: &Path) -> anyhow::Result<Option<Vec<u8>>> {
    Ok(None)
}
