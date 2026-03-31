use std::sync::{Mutex, MutexGuard};

pub fn lock_mutex<T>(mutex: &Mutex<T>) -> MutexGuard<'_, T> {
    #[cfg(not(target_arch = "wasm32"))]
    return mutex.lock().unwrap();

    // Can't casually lock on main thread in web, so let's do this simple spinlock
    #[cfg(target_arch = "wasm32")]
    loop {
        match mutex.try_lock() {
            Ok(guard) => return guard,
            Err(_) => {}
        }
    }
}
