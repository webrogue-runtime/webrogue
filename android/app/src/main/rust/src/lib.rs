use std::io::Write;
use std::sync::{Arc, Mutex};
use webrogue_runtime::wasi_common::{
    file::{FdFlags, FileType, WasiFile},
    Error, ErrorExt,
};
use webrogue_runtime::wiggle;
use webrogue_runtime::WasiFactory;

pub struct Stdout {
    output: Arc<Mutex<Vec<u8>>>,
}

extern "C" {
    fn webrogue_android_print(str: *const std::ffi::c_char, len: usize);
    fn webrogue_android_path() -> *const std::ffi::c_char;
}
#[wiggle::async_trait]
impl WasiFile for Stdout {
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
    #[cfg(unix)]
    fn pollable(&self) -> Option<rustix::fd::BorrowedFd> {
        None
    }
    #[cfg(windows)]
    fn pollable(&self) -> Option<io_extras::os::windows::RawHandleOrSocket> {
        None
    }

    async fn get_filetype(&self) -> Result<FileType, Error> {
        Ok(FileType::CharacterDevice)
    }
    async fn get_fdflags(&self) -> Result<FdFlags, Error> {
        Ok(FdFlags::APPEND)
    }
    async fn write_vectored<'a>(&self, bufs: &[std::io::IoSlice<'a>]) -> Result<u64, Error> {
        let mut n: usize = 0;
        let mut guard = self.output.lock().unwrap();
        for buf in bufs {
            n += guard.write(buf)?;
        }
        loop {
            match guard.iter().enumerate().find(|(_, x)| **x == b'\n') {
                Some((newline_pos, _)) => {
                    // let slice = &(*guard)[..newline_pos];
                    // let str = String::from_utf8_lossy(slice);

                    // println!("{}", str);
                    unsafe {
                        webrogue_android_print(
                            (*guard).as_ptr() as *const std::ffi::c_char,
                            newline_pos,
                        );
                    }
                    *guard = (*guard)[(newline_pos + 1)..].to_vec();
                }
                None => {
                    break;
                }
            }
        }
        Ok(n as u64)
    }
    async fn write_vectored_at<'a>(
        &self,
        _bufs: &[std::io::IoSlice<'a>],
        _offset: u64,
    ) -> Result<u64, Error> {
        Err(Error::seek_pipe())
    }
    async fn seek(&self, _pos: std::io::SeekFrom) -> Result<u64, Error> {
        Err(Error::seek_pipe())
    }
    async fn set_times(
        &self,
        _atime: Option<webrogue_runtime::wasi_common::SystemTimeSpec>,
        _mtime: Option<webrogue_runtime::wasi_common::SystemTimeSpec>,
    ) -> Result<(), Error> {
        Ok(())
    }
}

#[cfg(feature = "backend_wasmtime")]
use webrogue_backend_wasmtime::{Backend, make_funcs};

#[cfg(feature = "backend_v8")]
use webrogue_backend_v8::{Backend, make_funcs};

make_funcs!({
    "wasi_snapshot_preview1": {
        module: webrogue_wasi::wasi_snapshot_preview1
    },
    "webrogue_gl": {
        module: webrogue_gl::api
    },
    "webrogue_gfx": {
        module: webrogue_gfx
    }
});

fn main() -> anyhow::Result<()> {
    let lifecycle = webrogue_runtime::Lifecycle::new();

    let wasi_factory = webrogue_wasi_sync::WasiFactory::new();
    let mut wasi = wasi_factory.make();

    wasi_factory.add_actual_dir(&mut wasi, std::env::current_dir()?, "/");

    let vec: Vec<u8> = vec![];
    let output_data = Arc::new(Mutex::new(vec));
    wasi.set_stdout(Box::new(Stdout {
        output: output_data.clone(),
    }));
    wasi.set_stderr(Box::new(Stdout {
        output: output_data.clone(),
    }));

    // webrogue_std_stream_os::bind_streams(&mut wasi);
    let backend = Backend::new();

    
    let wrapp_path = unsafe { std::ffi::CStr::from_ptr(webrogue_android_path()).to_str().unwrap().to_owned() };
    
    let reader = webrogue_runtime::wrapp::WrappHandle::from_file_path(std::path::PathBuf::from(wrapp_path))?;

    let mut webrogue_gfx_context =
        webrogue_gfx::Context::new(Box::new(webrogue_gfx_ffi::make_system));
    let mut webrogue_gl_context = webrogue_gl::api::Context::new(&mut webrogue_gfx_context);
    lifecycle.run(
        backend,
        make_imports(),
        make_context_vec(
            &mut wasi,
            &mut webrogue_gl_context,
            &mut webrogue_gfx_context,
        ),
        reader,
    )?;

    Ok(())
}

#[no_mangle]
pub unsafe extern "C" fn webrogue_main() {
    main().unwrap();
}
