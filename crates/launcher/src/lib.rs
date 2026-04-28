mod webview;
pub use webview::build_webview;
mod server;
pub use server::run_server;
mod mailbox;
mod stored_config;
pub use mailbox::{Mailbox, MailboxInternal};
mod debug_connection;
mod launcher_config;
pub use launcher_config::LauncherConfig;
mod api_base_path;

#[cfg(feature = "winit")]
mod winit_app;
#[cfg(feature = "winit")]
pub use winit_app::*;
#[cfg(feature = "winit")]
mod winit_mailbox;
#[cfg(feature = "winit")]
pub use winit_mailbox::*;

pub fn install_default_crypto_provider() {
    let _ = rustls::crypto::aws_lc_rs::default_provider().install_default();
}
