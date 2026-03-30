use wasm_bindgen::JsCast;
use wasm_bindgen_futures::JsFuture;
use web_sys::{js_sys, FileSystemReadWriteOptions};

struct ReadRequest {
    result_tx: std::sync::mpsc::Sender<Vec<u8>>,
    pos: u64,
    len: u64,
}

pub struct SyncReader {
    pos: u64,
    len: u64,
    request_tx: std::sync::mpsc::Sender<ReadRequest>,
}

impl std::io::Read for SyncReader {
    fn read(&mut self, buf: &mut [u8]) -> std::io::Result<usize> {
        let (result_tx, result_rx) = std::sync::mpsc::channel();
        self.request_tx
            .send(ReadRequest {
                result_tx,
                pos: self.pos,
                len: buf.len() as u64,
            })
            .unwrap();
        let result = result_rx.recv().unwrap();

        buf[..(result.len())].copy_from_slice(&result);

        self.pos += result.len() as u64;
        Ok(result.len())
    }
}

impl std::io::Seek for SyncReader {
    fn seek(&mut self, pos: std::io::SeekFrom) -> std::io::Result<u64> {
        let new_pos = match pos {
            std::io::SeekFrom::Start(offset) => offset,
            std::io::SeekFrom::End(offset) => (self.len as i64 + offset) as u64,
            std::io::SeekFrom::Current(offset) => (self.pos as i64 + offset) as u64,
        };
        self.pos = new_pos;

        Ok(new_pos)
    }
}

impl webrogue_wrapp::Reader for SyncReader {}

impl SyncReader {
    pub fn new(file_name: String) -> Self {
        let (reader_tx, reader_rx) = std::sync::mpmc::channel();

        wasm_thread::Builder::new()
            .name("wrapp-io".to_owned())
            .spawn_async(async move || {
                let worker = js_sys::global()
                    .dyn_into::<web_sys::WorkerGlobalScope>()
                    .unwrap();
                let opfs_root = JsFuture::from(worker.navigator().storage().get_directory())
                    .await
                    .unwrap()
                    .dyn_into::<web_sys::FileSystemDirectoryHandle>()
                    .unwrap();

                let file_handle =
                    JsFuture::from(opfs_root.get_file_handle_with_options(&file_name, &{
                        let options = web_sys::FileSystemGetFileOptions::new();
                        options.set_create(true);
                        options
                    }))
                    .await
                    .unwrap()
                    .dyn_into::<web_sys::FileSystemFileHandle>()
                    .unwrap();

                let sync_access = JsFuture::from(file_handle.create_sync_access_handle())
                    .await
                    .unwrap()
                    .dyn_into::<web_sys::FileSystemSyncAccessHandle>()
                    .unwrap();

                // let vfs_builder = webrogue_wrapp::WrappVFSBuilder::from_vec(bytes).unwrap();
                let (request_tx, request_rx) = std::sync::mpsc::channel();
                reader_tx
                    .send(SyncReader {
                        request_tx,
                        pos: 0,
                        len: sync_access.get_size().unwrap() as u64,
                    })
                    .unwrap();
                let options = FileSystemReadWriteOptions::new();
                while let Ok(request) = request_rx.recv() {
                    let mut data = vec![0u8; request.len as usize];
                    options.set_at(request.pos as f64);
                    let read = sync_access
                        .read_with_u8_array_and_options(&mut data, &options)
                        .unwrap() as usize;
                    data.truncate(read);
                    request.result_tx.send(data).unwrap();
                }
            })
            .unwrap();

        reader_rx.recv().unwrap()
    }
}
