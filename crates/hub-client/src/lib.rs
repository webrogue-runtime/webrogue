pub mod debug_connection;
pub mod debug_messages;
pub mod ws_messages;

pub use webrogue_hub_client_openapi as openapi;

pub static WS_BASE_ADDR: &str = "wss://api.webrogue.dev";
pub static HTTP_BASE_ADDR: &str = "https://api.webrogue.dev";
