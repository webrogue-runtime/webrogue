pub mod real;
pub mod wrapp;

pub trait IFilePosition: Clone + Sync + Send {
    fn get_size(&self) -> usize;
}
pub trait IFileReader: std::io::Seek + std::io::Read + Sync + Send {}
pub trait IVFSHandle<FilePosition: IFilePosition, FileReader: IFileReader>:
    Clone + Sync + Send
{
    fn get_index(&self) -> &std::collections::HashMap<String, FilePosition>;
    fn open_pos(&self, position: FilePosition) -> FileReader;

    fn open_file(&self, path: &str) -> Option<FileReader> {
        self.get_index()
            .get(path)
            .map(|pos| self.open_pos(pos.clone()))
    }
}
