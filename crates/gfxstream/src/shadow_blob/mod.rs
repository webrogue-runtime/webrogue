#[cfg(shadow_blob)]
mod hash_based_blob;
#[cfg(signal_based_shadow_blob)]
mod signal_based_blob;
pub(crate) mod utils;
pub use utils::get_segfault_addr;

#[cfg(shadow_blob)]
mod functions {
    use std::sync::atomic::{AtomicBool, AtomicIsize, Ordering};

    static SHADOW_BLOB_IMPL: AtomicIsize = AtomicIsize::new(-1);
    static IS_EXTERNAL_SIGNAL_HANDLER_INSTALLED: AtomicBool = AtomicBool::new(false);

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

    pub fn init() {
        #[cfg(signal_based_shadow_blob)]
        if IS_EXTERNAL_SIGNAL_HANDLER_INSTALLED.load(Ordering::SeqCst) {
            ShadowBlobImpl::Signal
        } else if super::signal_based_blob::install_signal_handler() {
            ShadowBlobImpl::Signal
        } else {
            ShadowBlobImpl::Hash
        }
        .set();
        #[cfg(not(signal_based_shadow_blob))]
        ShadowBlobImpl::Hash.set();
        match ShadowBlobImpl::get() {
            ShadowBlobImpl::Hash => super::hash_based_blob::init(),
            #[cfg(signal_based_shadow_blob)]
            ShadowBlobImpl::Signal => super::signal_based_blob::init(),
        }
    }

    pub fn external_signal_handler_installed() {
        IS_EXTERNAL_SIGNAL_HANDLER_INSTALLED.store(true, Ordering::SeqCst);
    }

    pub fn flush_all() {
        match ShadowBlobImpl::get() {
            ShadowBlobImpl::Hash => super::hash_based_blob::flush_all(),
            #[cfg(signal_based_shadow_blob)]
            ShadowBlobImpl::Signal => super::signal_based_blob::flush_all(),
        }
    }

    pub fn handle_segfault(segfault_addr: *const ()) -> bool {
        match ShadowBlobImpl::get() {
            ShadowBlobImpl::Hash => super::hash_based_blob::handle_segfault(segfault_addr),
            #[cfg(signal_based_shadow_blob)]
            ShadowBlobImpl::Signal => super::signal_based_blob::handle_segfault(segfault_addr),
        }
    }
}

#[cfg(not(shadow_blob))]
mod functions {
    pub fn init() {}
    pub fn flush_all() {}
    pub fn handle_segfault(_segfault_addr: *const ()) -> bool {
        false
    }
    pub fn external_signal_handler_installed() {}
}

pub use functions::*;
