mod event_sink;
pub mod events;
mod interface;
#[cfg(feature = "wasmtime")]
mod vulkan_entry;
#[cfg(feature = "wasmtime")]
mod wasmtime;

pub use event_sink::{EventSink, EventStream};
pub use interface::{AbstractSystem, AbstractWindow, SurfaceKind, System, Window};
#[cfg(feature = "wasmtime")]
pub use vulkan_entry::load_vulkan_entry;
#[cfg(feature = "wasmtime")]
pub use wasmtime::{GFXCtxView, GFXView, add_to_linker};

#[cfg(all(feature = "wasmtime", feature = "vulkan"))]
pub mod shadow_blob {
    pub use webrogue_virgl::shadow_blob::{
        external_signal_handler_installed, get_segfault_addr, handle_segfault,
    };
}

#[cfg(all(feature = "wasmtime", not(feature = "vulkan")))]
pub mod shadow_blob {
    pub fn external_signal_handler_installed() {}

    pub fn handle_segfault(_segfault_addr: *const ()) -> bool {
        false
    }

    pub unsafe fn get_segfault_addr<SIGNUM, SIGINFO>(
        _signum: SIGNUM,
        _siginfo: SIGINFO,
    ) -> Option<*const ()> {
        None
    }
}

#[cfg(feature = "vulkan")]
pub mod virgl {
    pub use webrogue_virgl::{ContextContainer, Renderer, SystemProxy};
}
