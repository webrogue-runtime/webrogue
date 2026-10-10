mod hash_based_blob;
#[cfg(signal_based_shadow_blob)]
mod signal_based_blob;
pub(crate) mod utils;
use std::sync::atomic::{AtomicBool, AtomicIsize, Ordering};
pub use utils::get_segfault_addr;

static SHADOW_BLOB_IMPL: AtomicIsize = AtomicIsize::new(-1);
static IS_EXTERNAL_SIGNAL_HANDLER_INSTALLED: AtomicBool = AtomicBool::new(false);

enum ShadowBlobImpl {
    Hash,
    #[cfg(signal_based_shadow_blob)]
    Signal,
}

impl ShadowBlobImpl {
    fn get() -> Option<Self> {
        match SHADOW_BLOB_IMPL.load(Ordering::Relaxed) {
            -1 => None,
            1 => Some(Self::Hash),
            #[cfg(signal_based_shadow_blob)]
            2 => Some(Self::Signal),
            _ => unreachable!(),
        }
    }

    fn set(&self) {
        SHADOW_BLOB_IMPL.store(
            match self {
                ShadowBlobImpl::Hash => 1,
                #[cfg(signal_based_shadow_blob)]
                ShadowBlobImpl::Signal => 2,
            },
            Ordering::SeqCst,
        );
    }
}

pub fn init() {
    #[cfg(signal_based_shadow_blob)]
    if IS_EXTERNAL_SIGNAL_HANDLER_INSTALLED.load(Ordering::SeqCst) {
        ShadowBlobImpl::Signal
    } else if signal_based_blob::install_signal_handler() {
        ShadowBlobImpl::Signal
    } else {
        ShadowBlobImpl::Hash
    }
    .set();
    #[cfg(not(signal_based_shadow_blob))]
    ShadowBlobImpl::Hash.set();
    match ShadowBlobImpl::get() {
        Some(ShadowBlobImpl::Hash) => hash_based_blob::init(),
        #[cfg(signal_based_shadow_blob)]
        Some(ShadowBlobImpl::Signal) => signal_based_blob::init(),
        None => unreachable!(),
    }
}

pub fn external_signal_handler_installed() {
    IS_EXTERNAL_SIGNAL_HANDLER_INSTALLED.store(true, Ordering::SeqCst);
}

pub fn flush_all() {
    match ShadowBlobImpl::get() {
        Some(ShadowBlobImpl::Hash) => hash_based_blob::flush_all(),
        #[cfg(signal_based_shadow_blob)]
        Some(ShadowBlobImpl::Signal) => signal_based_blob::flush_all(),
        None => {}
    }
}

pub fn handle_segfault(segfault_addr: *const ()) -> bool {
    match ShadowBlobImpl::get() {
        Some(ShadowBlobImpl::Hash) => hash_based_blob::handle_segfault(segfault_addr),
        #[cfg(signal_based_shadow_blob)]
        Some(ShadowBlobImpl::Signal) => signal_based_blob::handle_segfault(segfault_addr),
        None => false,
    }
}

pub fn register_blob(vm_ptr: *const (), size: usize, blob_id: u64) {
    // TODO get host_blob size too
    let host_ptr = unsafe { crate::bindings::webrogue_get_host_blob(blob_id) } as *const ();
    if host_ptr.is_null() {
        return;
    }
    match ShadowBlobImpl::get() {
        Some(ShadowBlobImpl::Hash) => {
            hash_based_blob::register_blob(vm_ptr, size, host_ptr, blob_id)
        }
        #[cfg(signal_based_shadow_blob)]
        Some(ShadowBlobImpl::Signal) => {
            signal_based_blob::register_blob(vm_ptr, size, host_ptr, blob_id)
        }
        None => unreachable!(),
    }
}

pub fn deregister_blob(blob_id: u64) {
    match ShadowBlobImpl::get() {
        Some(ShadowBlobImpl::Hash) => hash_based_blob::deregister_blob(blob_id),
        #[cfg(signal_based_shadow_blob)]
        Some(ShadowBlobImpl::Signal) => signal_based_blob::deregister_blob(blob_id),
        None => unreachable!(),
    }
}
