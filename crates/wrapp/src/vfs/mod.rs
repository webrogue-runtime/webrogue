pub mod real;
pub mod wrapp;

pub trait IFilePosition: Clone + Sync + Send + std::fmt::Display {
    fn get_size(&self) -> usize;
}
pub trait IFileReader: std::io::Seek + std::io::Read + Sync + Send {}
pub trait IVFSHandle<FilePosition: IFilePosition, FileReader: IFileReader>:
    Clone + Sync + Send
{
    fn get_index(&self) -> &std::collections::HashMap<String, FilePosition>;
    fn open_pos(&self, position: FilePosition) -> anyhow::Result<FileReader>;

    fn open_file(&self, path: &str) -> anyhow::Result<Option<FileReader>> {
        let Some(pos) = self.get_index().get(path) else {
            return Ok(None);
        };

        Ok(Some(self.open_pos(pos.clone())?))
    }
}
pub trait IVFSBuilder<
    FilePosition: IFilePosition,
    FileReader: IFileReader,
    VFSHandle: IVFSHandle<FilePosition, FileReader>,
>
{
    fn into_vfs(self) -> anyhow::Result<VFSHandle>;
    fn config(&mut self) -> anyhow::Result<&crate::config::Config>;
    fn get_uncompressed(&mut self, name: &str) -> anyhow::Result<Option<Vec<u8>>>;
}
