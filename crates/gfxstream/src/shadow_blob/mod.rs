#[cfg(signal_bases_shadow_blob)]
mod signal_based_blob;
#[cfg(signal_bases_shadow_blob)]
pub use signal_based_blob::*;

#[cfg(not(signal_bases_shadow_blob))]
mod stub_blob;
#[cfg(not(signal_bases_shadow_blob))]
pub use stub_blob::*;
