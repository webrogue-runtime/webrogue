// This file has been shamelessly copied from https://github.com/bytecodealliance/wasmtime/tree/main/crates/wasi-threads
use std::sync::Arc;

pub struct ThreadsContext {
    tid: std::sync::atomic::AtomicI32,
    pub imports: Arc<crate::imports::Imports>,
}

extern "C" {
    fn wr_thread_start_listening() -> u64;
    fn wr_thread_send_message(tid: u64);
    fn wr_rs_thread_wait(context: *mut std::ffi::c_void, json_ptr: *const u8);
}

fn exec_func(func_name: &str, arg0: i32, arg1: i32) {
    unsafe {
        let mut func_name = func_name.as_bytes().to_vec();
        func_name.push(0);
        crate::ffi::wr_exec_func_ii(func_name.as_ptr(), arg0, arg1);
    }
}

impl ThreadsContext {
    pub fn new(imports: crate::imports::Imports) -> Self {
        let tid = std::sync::atomic::AtomicI32::new(0);
        Self {
            tid,
            imports: Arc::new(imports),
        }
    }

    pub fn spawn(
        &self,
        store: crate::context::Store,
        thread_start_arg: i32,
    ) -> anyhow::Result<i32> {
        let wasi_thread_id = self.next_thread_id();
        if wasi_thread_id.is_none() {
            return Ok(-1);
        }
        let wasi_thread_id = wasi_thread_id.unwrap();

        let builder = std::thread::Builder::new().name(format!("wasi-thread-{wasi_thread_id}"));
        let imports = self.imports.as_ref().clone();
        let (tx, rx) = std::sync::mpsc::channel::<u64>();
        builder.spawn(move || {
            let mut context = crate::context::Context::new(imports);
            context.store = Some(store);
            tx.send(unsafe { wr_thread_start_listening() }).unwrap();

            let mut jsonptr = context.imports.to_json().as_bytes().to_vec();
            jsonptr.push(0);

            unsafe {
                wr_rs_thread_wait(
                    ((&mut context) as *mut crate::context::Context) as *mut std::ffi::c_void,
                    jsonptr.as_ptr(),
                );
            }
            exec_func("wasi_thread_start", wasi_thread_id, thread_start_arg);
            // std::thread::sleep(std::time::Duration::from_secs(1000));
            // if epoch_interruption {
            //     store.epoch_deadline_trap();
            //     store.set_epoch_deadline(1);
            // }
            // let instance = instance_pre.instantiate(&mut store)?;
            // let thread_entry_point = instance
            //     .get_typed_func::<(i32, i32), ()>(&mut store, WASI_ENTRY_POINT)
            //     .unwrap();
            // let res =
            //     thread_entry_point.call(&mut store, (wasi_thread_id, thread_start_arg));
            // match res {
            //     Ok(_) => Ok(()),
            //     Err(e) => Err(e),
            // }
        })?;
        let tid = rx.recv().unwrap();
        unsafe { wr_thread_send_message(tid) };

        Ok(wasi_thread_id)
    }

    fn next_thread_id(&self) -> Option<i32> {
        match self.tid.fetch_update(
            std::sync::atomic::Ordering::Relaxed,
            std::sync::atomic::Ordering::Relaxed,
            |v| match v {
                ..=0x1ffffffe => Some(v + 1),
                _ => None,
            },
        ) {
            Ok(v) => Some(v + 1),
            Err(_) => None,
        }
    }
}
