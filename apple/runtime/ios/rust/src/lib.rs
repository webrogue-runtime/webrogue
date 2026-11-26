#[cfg(feature = "launcher")]
pub mod launcher;
#[cfg(feature = "runner")]
pub mod runner;

/// Needed to make clippy stfu, as webrogue-gfx dependency is needed to enable "static-vk" feature
fn _unused() {
    let _ = webrogue_gfx::SurfaceKind::Vulkan;
}
