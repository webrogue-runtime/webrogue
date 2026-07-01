mod hash_based_blob;
#[cfg(signal_based_shadow_blob)]
mod signal_based_blob;
#[cfg(signal_based_shadow_blob)]
pub use signal_based_blob::get_segfault_addr;
pub(crate) mod utils;

use std::sync::atomic::{AtomicIsize, Ordering};

static SHADOW_BLOB_IMPL: AtomicIsize = AtomicIsize::new(-1);

enum ShadowBlobImpl {
    Hash,
    #[cfg(signal_based_shadow_blob)]
    Signal,
}

impl ShadowBlobImpl {
    fn get() -> Self {
        match SHADOW_BLOB_IMPL.load(Ordering::Relaxed) {
            1 => Self::Hash,
            #[cfg(signal_based_shadow_blob)]
            2 => Self::Signal,
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

pub fn init(debug: bool) {
    #[cfg(signal_based_shadow_blob)]
    if debug {
        ShadowBlobImpl::Hash
    } else {
        ShadowBlobImpl::Signal
    }
    .set();
    #[cfg(not(signal_based_shadow_blob))]
    ShadowBlobImpl::Hash.set();
    match ShadowBlobImpl::get() {
        ShadowBlobImpl::Hash => hash_based_blob::init(),
        #[cfg(signal_based_shadow_blob)]
        ShadowBlobImpl::Signal => signal_based_blob::init(),
    }
}

pub fn flush_all() {
    match ShadowBlobImpl::get() {
        ShadowBlobImpl::Hash => hash_based_blob::flush_all(),
        #[cfg(signal_based_shadow_blob)]
        ShadowBlobImpl::Signal => signal_based_blob::flush_all(),
    }
}

pub fn handle_segfault(segfault_addr: *const ()) -> bool {
    match ShadowBlobImpl::get() {
        ShadowBlobImpl::Hash => hash_based_blob::handle_segfault(segfault_addr),
        #[cfg(signal_based_shadow_blob)]
        ShadowBlobImpl::Signal => signal_based_blob::handle_segfault(segfault_addr),
    }
}

// #[cfg(not(signal_based_shadow_blob))]
// mod stub_blob;
// #[cfg(not(signal_based_shadow_blob))]
// pub use stub_blob::*;
