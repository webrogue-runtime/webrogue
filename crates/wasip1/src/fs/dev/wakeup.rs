use webrogue_wasi_common::{file::FileType, ErrorExt as _, WasiFile};

use webrogue_wasi_common::ErrorExt as _;

pub struct File {
    tx: tokio::sync::watch::Sender<bool>,
    rx: tokio::sync::watch::Receiver<bool>,
}

impl File {
    pub fn new() -> Result<Self, webrogue_wasi_common::Error> {
        let (tx, rx) = tokio::sync::watch::channel(false);

        Ok(Self { tx, rx })
    }

    pub(super) fn acknowledge(&self) -> () {
        self.tx.send(false).unwrap()
    }

    pub(super) fn signal(&self) -> () {
        self.tx.send(true).unwrap()
    }

    pub(super) async fn wait(&self) -> () {
        let mut rx = self.rx.clone();

        while !*rx.borrow() {
            // This blocks until the sender updates the state
            rx.changed().await.unwrap();
        }
    }
}

#[async_trait::async_trait]
impl WasiFile for File {
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }

    async fn get_filetype(&self) -> Result<FileType, webrogue_wasi_common::Error> {
        Ok(FileType::Pipe)
    }

    async fn read_vectored<'a>(
        &self,
        _bufs: &mut [std::io::IoSliceMut<'a>],
    ) -> Result<u64, webrogue_wasi_common::Error> {
        self.acknowledge();
        Ok(0)
    }

    async fn write_vectored<'a>(
        &self,
        _bufs: &[std::io::IoSlice<'a>],
    ) -> Result<u64, webrogue_wasi_common::Error> {
        self.signal();
        Ok(_bufs.iter().map(|slice| slice.len()).sum::<usize>() as u64)
    }

    async fn readable(&self) -> Result<(), webrogue_wasi_common::Error> {
        self.wait();
        Ok(())
    }

    async fn writable(&self) -> Result<(), webrogue_wasi_common::Error> {
        Ok(())
    }
}

// #[cfg(test)]
// mod tests {
//     use crate::fs::dev::wakeup::File;
//     use std::{
//         sync::{
//             atomic::{AtomicUsize, Ordering},
//             mpsc, Arc,
//         },
//         thread,
//     };

//     #[cfg(unix)]
//     fn wait(wakeup: &Arc<File>) {
//         use rustix::event::{poll, PollFd, PollFlags};

//         let fd = PollFd::from_borrowed_fd(wakeup.pollable(), PollFlags::IN);
//         let a = poll(&mut [fd], None).unwrap();
//         assert_eq!(a, 1);
//     }

//     #[cfg(windows)]
//     fn wait(wakeup: &Arc<File>) {
//         use windows_sys::Win32::System::Threading::{WaitForSingleObject, INFINITE};

//         let handle = wakeup.pollable();
//         unsafe {
//             WaitForSingleObject(handle.as_raw_handle().unwrap(), INFINITE);
//         }
//     }

//     fn invoke_iters() -> u32 {
//         1 + rand::random::<u32>() % 10
//     }

//     #[test]
//     fn blocks() {
//         let wakeup = Arc::new(File::new().unwrap());
//         let thrd_wakeup = wakeup.clone();
//         let counter = Arc::new(AtomicUsize::new(0));
//         let thrd_counter = counter.clone();
//         let (tx, rx) = mpsc::channel::<()>();

//         const ITERS: usize = 10000;

//         let thread = thread::spawn(move || {
//             for i in 0..ITERS {
//                 assert_eq!(thrd_counter.fetch_add(1, Ordering::SeqCst), 0 + i * 2);
//                 // TODO figure out why invoking signal() multiple times without acknowledge()
//                 // inbetween has a small chance to cause data race on Linux.
//                 thrd_wakeup.signal().unwrap();
//                 rx.recv().unwrap();
//             }
//         });

//         for i in 0..ITERS {
//             for _ in 0..invoke_iters() {
//                 wait(&wakeup);
//             }
//             assert_eq!(counter.fetch_add(1, Ordering::SeqCst), 1 + i * 2);
//             for _ in 0..invoke_iters() {
//                 wakeup.acknowledge().unwrap();
//             }
//             tx.send(()).unwrap();
//         }

//         thread.join().unwrap();
//     }

//     #[test]
//     fn many_signals() {
//         let wakeup = Arc::new(File::new().unwrap());
//         let thrd_wakeup = wakeup.clone();
//         let counter = Arc::new(AtomicUsize::new(0));
//         let thrd_counter = counter.clone();
//         let (tx, rx) = mpsc::channel::<()>();
//         let (tx2, rx2) = mpsc::channel::<()>();

//         const ITERS: usize = 10000;

//         let thread = thread::spawn(move || {
//             for i in 0..ITERS {
//                 assert_eq!(thrd_counter.fetch_add(1, Ordering::SeqCst), 0 + i * 2);
//                 // TODO figure out why invoking signal() multiple times without acknowledge()
//                 // inbetween has a small chance to cause data race on Linux.
//                 for _ in 0..invoke_iters() {
//                     thrd_wakeup.signal().unwrap();
//                 }
//                 tx.send(()).unwrap();
//                 rx2.recv().unwrap();
//             }
//         });

//         for i in 0..ITERS {
//             wait(&wakeup);
//             rx.recv().unwrap();
//             assert_eq!(counter.fetch_add(1, Ordering::SeqCst), 1 + i * 2);
//             wakeup.acknowledge().unwrap();
//             tx2.send(()).unwrap();
//         }

//         thread.join().unwrap();
//     }
// }
