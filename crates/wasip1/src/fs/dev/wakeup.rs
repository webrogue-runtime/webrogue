use wasi_common::{file::FileType, ErrorExt as _, WasiFile};

#[cfg(unix)]
mod unix {
    use std::os::fd::{AsFd, OwnedFd};

    use rustix::io::{self, Errno};
    use wasi_common::ErrorExt as _;

    pub struct File {
        read_fd: OwnedFd,
        write_fd: OwnedFd,
    }

    impl File {
        pub fn new() -> Result<Self, wasi_common::Error> {
            use rustix::pipe::{pipe_with, PipeFlags};

            let (read_fd, write_fd) = pipe_with(PipeFlags::NONBLOCK | PipeFlags::CLOEXEC)
                .map_err(|_err| wasi_common::Error::not_supported())?;
            Ok(Self { read_fd, write_fd })
        }

        pub(super) fn acknowledge(&self) -> io::Result<()> {
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

        pub(super) fn signal(&self) -> io::Result<()> {
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
    pub struct File {}

    impl File {
        pub fn new() -> Result<Self, wasi_common::Error> {
            unimplemented!()
        }

        pub(super) fn acknowledge(&self) -> Result<(), ()> {
            unimplemented!()
        }

        pub(super) fn signal(&self) -> Result<(), ()> {
            unimplemented!()
        }

        pub(super) fn pollable(&self) -> io_extras::os::windows::RawHandleOrSocket {
            unimplemented!()
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
        Some(File::pollable(&self))
    }

    #[cfg(windows)]
    fn pollable(&self) -> Option<io_extras::os::windows::RawHandleOrSocket> {
        Some(File::pollable(&self))
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
    use rand::Rng;
    use std::{
        sync::{
            atomic::{AtomicUsize, Ordering},
            mpsc, Arc,
        },
        thread,
    };

    #[test]
    fn blocks() {
        let wakeup = Arc::new(File::new().unwrap());
        let thrd_wakeup = wakeup.clone();
        let counter = Arc::new(AtomicUsize::new(0));
        let thrd_counter = counter.clone();
        let (tx, rx) = mpsc::channel::<()>();

        #[cfg(unix)]
        fn wait(wakeup: &Arc<File>) {
            use rustix::event::{poll, PollFd, PollFlags};

            let fd = PollFd::from_borrowed_fd(wakeup.pollable(), PollFlags::IN);
            let a = poll(&mut [fd], None).unwrap();
            assert_eq!(a, 1);
        }

        const ITERS: usize = 10000;

        fn invoke_iters() -> u32 {
            1 + rand::thread_rng().gen::<u32>() % 10
        }

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
}
