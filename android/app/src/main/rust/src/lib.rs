extern "C" {
    fn webrogue_android_print(str: *const std::ffi::c_char, len: usize);
    fn webrogue_android_path() -> *const std::ffi::c_char;
}

#[derive(Debug)]
struct Stdout {}

impl Stdout {
    fn new() -> Self {
        Self {}
    }
}

impl wasmer_wasix::VirtualFile for Stdout {
    fn last_accessed(&self) -> u64 {
        0
    }

    fn last_modified(&self) -> u64 {
        0
    }

    fn created_time(&self) -> u64 {
        0
    }

    fn size(&self) -> u64 {
        0
    }

    fn set_len(&mut self, _new_size: u64) -> wasmer_wasix::virtual_fs::Result<()> {
        Ok(())
    }

    fn unlink(&mut self) -> wasmer_wasix::virtual_fs::Result<()> {
        Ok(())
    }

    fn get_special_fd(&self) -> Option<u32> {
        Some(1)
    }

    fn poll_read_ready(
        self: std::pin::Pin<&mut Self>,
        _cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<std::io::Result<usize>> {
        std::task::Poll::Ready(Ok(0))
    }

    fn poll_write_ready(
        self: std::pin::Pin<&mut Self>,
        _cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<std::io::Result<usize>> {
        std::task::Poll::Ready(Ok(1024))
    }
}

impl tokio::io::AsyncRead for Stdout {
    fn poll_read(
        self: std::pin::Pin<&mut Self>,
        _cx: &mut std::task::Context<'_>,
        _buf: &mut tokio::io::ReadBuf<'_>,
    ) -> std::task::Poll<std::io::Result<()>> {
        std::task::Poll::Ready(std::io::Result::Err(std::io::Error::new(
            std::io::ErrorKind::Other,
            "can not read from stdout",
        )))
    }
}

impl tokio::io::AsyncWrite for Stdout {
    fn poll_write(
        self: std::pin::Pin<&mut Self>,
        _cx: &mut std::task::Context<'_>,
        buf: &[u8],
    ) -> std::task::Poll<Result<usize, std::io::Error>> {
        unsafe {
            webrogue_android_print(buf.as_ptr() as *const std::ffi::c_char, buf.len());
        }
        std::task::Poll::Ready(Result::Ok(buf.len()))
    }

    fn poll_flush(
        self: std::pin::Pin<&mut Self>,
        _cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<Result<(), std::io::Error>> {
        std::task::Poll::Ready(Result::Ok(()))
    }

    fn poll_shutdown(
        self: std::pin::Pin<&mut Self>,
        _cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<Result<(), std::io::Error>> {
        std::task::Poll::Ready(Result::Ok(()))
    }
}

impl tokio::io::AsyncSeek for Stdout {
    fn start_seek(
        self: std::pin::Pin<&mut Self>,
        _position: std::io::SeekFrom,
    ) -> std::io::Result<()> {
        std::io::Result::Err(std::io::Error::new(
            std::io::ErrorKind::Other,
            "can not seek stdout",
        ))
    }

    fn poll_complete(
        self: std::pin::Pin<&mut Self>,
        _cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<std::io::Result<u64>> {
        std::task::Poll::Ready(std::io::Result::Err(std::io::Error::new(
            std::io::ErrorKind::Other,
            "can not seek stdout",
        )))
    }
}

fn main() -> anyhow::Result<()> {
    let container_path = unsafe {
        std::ffi::CStr::from_ptr(webrogue_android_path())
            .to_str()
            .unwrap()
            .to_owned()
    };

    webrogue_runtime::run(
        wasmer_package::utils::from_disk(std::path::PathBuf::from(container_path))?,
        Some(Box::new(Stdout::new())),
    )?;

    Ok(())
}

#[no_mangle]
pub unsafe extern "C" fn webrogue_main() {
    main().unwrap();
}
