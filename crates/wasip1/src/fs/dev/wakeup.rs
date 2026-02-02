use wasi_common::{file::FileType, ErrorExt as _, WasiFile};

#[cfg(unix)]
mod unix {
    use std::os::fd::AsFd;

    use wasi_common::ErrorExt as _;

    pub struct File {
        read_fd: std::os::fd::OwnedFd,
        write_fd: std::os::fd::OwnedFd,
    }

    impl File {
        pub fn new() -> Result<Self, wasi_common::Error> {
            // let (read_fd, write_fd) = rustix::pipe::pipe_with(rustix::pipe::PipeFlags::NONBLOCK | rustix::pipe::PipeFlags::CLOEXEC)
            //     .map_err(|_err| wasi_common::Error::not_supported())?;
            let (read_fd, write_fd) =
                rustix::pipe::pipe().map_err(|_err| wasi_common::Error::not_supported())?;
            for fd in [read_fd.as_fd(), read_fd.as_fd()] {
                rustix::io::fcntl_setfd(fd, rustix::io::FdFlags::CLOEXEC)
                    .map_err(|_err| wasi_common::Error::not_supported())?;
                rustix::io::ioctl_fionbio(fd, true)
                    .map_err(|_err| wasi_common::Error::not_supported())?;
            }
            Ok(Self { read_fd, write_fd })
        }

        pub(super) fn acknowledge(&self) -> rustix::io::Result<()> {
            use rustix::io::Errno;

            let mut buf = [0u8; 8];
            loop {
                match rustix::io::read(self.read_fd.as_fd(), &mut buf) {
                    Ok(0) => {
                        return Ok(());
                    }
                    Ok(_) | Err(Errno::INTR) => {}
                    Err(_) => {
                        return Ok(());
                    }
                }
            }
        }

        pub(super) fn signal(&self) -> rustix::io::Result<()> {
            use rustix::io::Errno;
            const LEN: usize = 1;
            loop {
                match rustix::io::write(self.write_fd.as_fd(), &[1u8; LEN]) {
                    Ok(0) | Err(Errno::INTR | Errno::WOULDBLOCK) => {}
                    Ok(_) | Err(_) => {
                        return Ok(());
                    }
                }
            }
        }

        pub(super) fn pollable(&self) -> rustix::fd::BorrowedFd<'_> {
            use std::os::fd::AsFd as _;

            self.read_fd.as_fd()
        }
    }
}

#[cfg(unix)]
pub use unix::File;

#[cfg(windows)]
mod windows {
    use std::{
        os::windows::io::{AsRawHandle, FromRawHandle, OwnedHandle},
        ptr::{null, null_mut},
    };

    use windows_sys::Win32::System::Threading::{CreateEventA, ResetEvent, SetEvent};

    pub struct File {
        handle: OwnedHandle,
    }

    impl File {
        pub fn new() -> Result<Self, wasi_common::Error> {
            let handle = unsafe { CreateEventA(null(), 1, 0, null()) };
            assert!(handle != null_mut());
            Ok(Self {
                handle: unsafe { OwnedHandle::from_raw_handle(handle) },
            })
        }

        pub(super) fn acknowledge(&self) -> Result<(), ()> {
            unsafe {
                ResetEvent(self.handle.as_raw_handle());
            }
            Ok(())
        }

        pub(super) fn signal(&self) -> Result<(), ()> {
            unsafe {
                SetEvent(self.handle.as_raw_handle());
            }
            Ok(())
        }

        pub(super) fn pollable(&self) -> io_extras::os::windows::RawHandleOrSocket {
            io_extras::os::windows::RawHandleOrSocket::unowned_from_raw_handle(
                self.handle.as_raw_handle(),
            )
        }
    }
}

#[cfg(windows)]
pub use windows::File;

#[async_trait::async_trait]
impl WasiFile for File {
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }

    async fn get_filetype(&self) -> Result<FileType, wasi_common::Error> {
        Ok(FileType::Pipe)
    }

    #[cfg(unix)]
    fn pollable(&self) -> Option<rustix::fd::BorrowedFd<'_>> {
        Some(File::pollable(self))
    }

    #[cfg(windows)]
    fn pollable(&self) -> Option<io_extras::os::windows::RawHandleOrSocket> {
        Some(File::pollable(self))
    }

    async fn read_vectored<'a>(
        &self,
        _bufs: &mut [std::io::IoSliceMut<'a>],
    ) -> Result<u64, wasi_common::Error> {
        self.acknowledge()
            .map_err(|_| wasi_common::Error::not_supported())?;
        Ok(0)
    }

    async fn write_vectored<'a>(
        &self,
        _bufs: &[std::io::IoSlice<'a>],
    ) -> Result<u64, wasi_common::Error> {
        self.signal()
            .map_err(|_| wasi_common::Error::not_supported())?;

        Ok(_bufs.iter().map(|slice| slice.len()).sum::<usize>() as u64)
    }
}

#[cfg(test)]
mod tests {
    use crate::fs::dev::wakeup::File;
    use std::{
        sync::{
            atomic::{AtomicUsize, Ordering},
            mpsc, Arc,
        },
        thread,
    };

    #[cfg(unix)]
    fn wait(wakeup: &Arc<File>) {
        use rustix::event::{poll, PollFd, PollFlags};

        let fd = PollFd::from_borrowed_fd(wakeup.pollable(), PollFlags::IN);
        let a = poll(&mut [fd], None).unwrap();
        assert_eq!(a, 1);
    }

    #[cfg(windows)]
    fn wait(wakeup: &Arc<File>) {
        use windows_sys::Win32::System::Threading::{WaitForSingleObject, INFINITE};

        let handle = wakeup.pollable();
        unsafe {
            WaitForSingleObject(handle.as_raw_handle().unwrap(), INFINITE);
        }
    }

    fn invoke_iters() -> u32 {
        1 + rand::random::<u32>() % 10
    }

    #[test]
    fn blocks() {
        let wakeup = Arc::new(File::new().unwrap());
        let thrd_wakeup = wakeup.clone();
        let counter = Arc::new(AtomicUsize::new(0));
        let thrd_counter = counter.clone();
        let (tx, rx) = mpsc::channel::<()>();

        const ITERS: usize = 10000;

        let thread = thread::spawn(move || {
            for i in 0..ITERS {
                assert_eq!(thrd_counter.fetch_add(1, Ordering::SeqCst), 0 + i * 2);
                // TODO figure out why invoking signal() multiple times without acknowledge()
                // inbetween has a small chance to cause data race on Linux.
                thrd_wakeup.signal().unwrap();
                rx.recv().unwrap();
            }
        });

        for i in 0..ITERS {
            for _ in 0..invoke_iters() {
                wait(&wakeup);
            }
            assert_eq!(counter.fetch_add(1, Ordering::SeqCst), 1 + i * 2);
            for _ in 0..invoke_iters() {
                wakeup.acknowledge().unwrap();
            }
            tx.send(()).unwrap();
        }

        thread.join().unwrap();
    }

    #[test]
    fn many_signals() {
        let wakeup = Arc::new(File::new().unwrap());
        let thrd_wakeup = wakeup.clone();
        let counter = Arc::new(AtomicUsize::new(0));
        let thrd_counter = counter.clone();
        let (tx, rx) = mpsc::channel::<()>();
        let (tx2, rx2) = mpsc::channel::<()>();

        const ITERS: usize = 10000;

        let thread = thread::spawn(move || {
            for i in 0..ITERS {
                assert_eq!(thrd_counter.fetch_add(1, Ordering::SeqCst), 0 + i * 2);
                // TODO figure out why invoking signal() multiple times without acknowledge()
                // inbetween has a small chance to cause data race on Linux.
                for _ in 0..invoke_iters() {
                    thrd_wakeup.signal().unwrap();
                }
                tx.send(()).unwrap();
                rx2.recv().unwrap();
            }
        });

        for i in 0..ITERS {
            wait(&wakeup);
            rx.recv().unwrap();
            assert_eq!(counter.fetch_add(1, Ordering::SeqCst), 1 + i * 2);
            wakeup.acknowledge().unwrap();
            tx2.send(()).unwrap();
        }

        thread.join().unwrap();
    }
}
