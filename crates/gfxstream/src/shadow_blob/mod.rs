#[cfg(signal_based_shadow_blob)]
mod signal_based_blob;
#[cfg(signal_based_shadow_blob)]
pub use signal_based_blob::*;

pub(crate) mod utils;

#[cfg(not(signal_based_shadow_blob))]
mod stub_blob;
#[cfg(not(signal_based_shadow_blob))]
pub use stub_blob::*;
