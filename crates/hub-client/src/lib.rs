pub mod api_base_path;
pub mod debug_connection;
pub mod debug_message_receiver;
pub mod debug_message_sender;
pub mod debug_messages;
pub mod ws_connection;
pub mod ws_messages;

pub use webrogue_hub_client_openapi as openapi;

// Re-exports for convenience
pub use ws_connection::{
    handle_websocket_message, wait_for_text_message_with_pings, MessageHandleResult, PING_INTERVAL,
};
pub use ws_messages::{ConnectDeviceWsCommand, ConnectDeviceWsEvent};
