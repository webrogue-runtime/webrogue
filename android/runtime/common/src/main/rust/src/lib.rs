use std::os::unix::io::FromRawFd;

extern "C" {
    fn webrogue_android_print(str: *const std::ffi::c_char, len: usize);
    fn webrogue_android_data_path() -> *const std::ffi::c_char;
    fn webrogue_android_container_fd() -> i32;
    fn webrogue_android_container_offset() -> i32;
    fn webrogue_android_container_size() -> i32;
}

// #[derive(Debug)]
// struct Stdout {}

// impl Stdout {
//     fn new() -> Self {
//         Self {}
//     }
// }

// impl wasmer_wasix::VirtualFile for Stdout {
//     fn last_accessed(&self) -> u64 {
//         0
//     }

//     fn last_modified(&self) -> u64 {
//         0
//     }

//     fn created_time(&self) -> u64 {
//         0
//     }

//     fn size(&self) -> u64 {
//         0
//     }

//     fn set_len(&mut self, _new_size: u64) -> wasmer_wasix::virtual_fs::Result<()> {
//         Ok(())
//     }

//     fn unlink(&mut self) -> wasmer_wasix::virtual_fs::Result<()> {
//         Ok(())
//     }

//     fn get_special_fd(&self) -> Option<u32> {
//         Some(1)
//     }

//     fn poll_read_ready(
//         self: std::pin::Pin<&mut Self>,
//         _cx: &mut std::task::Context<'_>,
//     ) -> std::task::Poll<std::io::Result<usize>> {
//         std::task::Poll::Ready(Ok(0))
//     }

//     fn poll_write_ready(
//         self: std::pin::Pin<&mut Self>,
//         _cx: &mut std::task::Context<'_>,
//     ) -> std::task::Poll<std::io::Result<usize>> {
//         std::task::Poll::Ready(Ok(1024))
//     }
// }

// impl tokio::io::AsyncRead for Stdout {
//     fn poll_read(
//         self: std::pin::Pin<&mut Self>,
//         _cx: &mut std::task::Context<'_>,
//         _buf: &mut tokio::io::ReadBuf<'_>,
//     ) -> std::task::Poll<std::io::Result<()>> {
//         std::task::Poll::Ready(std::io::Result::Err(std::io::Error::new(
//             std::io::ErrorKind::Other,
//             "can not read from stdout",
//         )))
//     }
// }

// impl tokio::io::AsyncWrite for Stdout {
//     fn poll_write(
//         self: std::pin::Pin<&mut Self>,
//         _cx: &mut std::task::Context<'_>,
//         buf: &[u8],
//     ) -> std::task::Poll<Result<usize, std::io::Error>> {
//         unsafe {
//             webrogue_android_print(buf.as_ptr() as *const std::ffi::c_char, buf.len());
//         }
//         std::task::Poll::Ready(Result::Ok(buf.len()))
//     }

//     fn poll_flush(
//         self: std::pin::Pin<&mut Self>,
//         _cx: &mut std::task::Context<'_>,
//     ) -> std::task::Poll<Result<(), std::io::Error>> {
//         std::task::Poll::Ready(Result::Ok(()))
//     }

//     fn poll_shutdown(
//         self: std::pin::Pin<&mut Self>,
//         _cx: &mut std::task::Context<'_>,
//     ) -> std::task::Poll<Result<(), std::io::Error>> {
//         std::task::Poll::Ready(Result::Ok(()))
//     }
// }

// impl tokio::io::AsyncSeek for Stdout {
//     fn start_seek(
//         self: std::pin::Pin<&mut Self>,
//         _position: std::io::SeekFrom,
//     ) -> std::io::Result<()> {
//         std::io::Result::Err(std::io::Error::new(
//             std::io::ErrorKind::Other,
//             "can not seek stdout",
//         ))
//     }

//     fn poll_complete(
//         self: std::pin::Pin<&mut Self>,
//         _cx: &mut std::task::Context<'_>,
//     ) -> std::task::Poll<std::io::Result<u64>> {
//         std::task::Poll::Ready(std::io::Result::Err(std::io::Error::new(
//             std::io::ErrorKind::Other,
//             "can not seek stdout",
//         )))
//     }
// }

fn main() -> anyhow::Result<()> {
    let data_path = unsafe {
        std::ffi::CStr::from_ptr(webrogue_android_data_path())
            .to_str()
            .unwrap()
            .to_owned()
    };

    let file = unsafe { std::fs::File::from_raw_fd(webrogue_android_container_fd()) };
    let offset = unsafe { webrogue_android_container_offset() } as u64;
    let size = unsafe { webrogue_android_container_size() } as u64;

    let mut builder = webrogue_runtime::WrappHandleBuilder::from_file_part(file, offset, size)?;

    let persistent_path = std::path::PathBuf::from(data_path)
        .join(".webrogue")
        .join(&builder.config()?.id)
        .join("persistent");
    webrogue_runtime::Config::from_builder(builder, persistent_path)?.run()?;

    Ok(())
}

#[no_mangle]
pub unsafe extern "C" fn webrogue_main() {
    match main() {
        Ok(_) => {}
        Err(e) => {
            println!("{}", e);
            let s = format!("{}", e);
            let b = s.as_bytes();
            unsafe {
                webrogue_android_print(b.as_ptr() as *const std::ffi::c_char, b.len());
            }
        }
    };
}
