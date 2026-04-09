pub mod debug_connection;
pub mod debug_messages;
pub mod ws_messages;

pub use webrogue_hub_client_openapi as openapi;

pub static WS_BASE_ADDR: &str = "ws://localhost:8080";
pub static HTTP_BASE_ADDR: &str = "http://localhost:8080";
