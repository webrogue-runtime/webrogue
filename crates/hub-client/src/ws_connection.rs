use std::time::Duration;

use futures_util::{SinkExt, StreamExt};
use tokio_tungstenite::{connect_async, tungstenite::Message};

use crate::ws_messages::{ConnectDeviceWsCommand, ConnectDeviceWsEvent};

pub const PING_INTERVAL: Duration = Duration::from_secs(60);

pub enum MessageHandleResult {
    Continue,
    TextBreak(String),
}

pub async fn handle_websocket_message<S>(
    msg: Message,
    write: &mut S,
) -> anyhow::Result<MessageHandleResult>
where
    S: SinkExt<Message> + Unpin,
    S::Error: Into<anyhow::Error>,
{
    match msg {
        Message::Text(utf8_bytes) => {
            tracing::debug!("Received text message from server");
            Ok(MessageHandleResult::TextBreak(utf8_bytes.to_string()))
        }
        Message::Binary(_bytes) => {
            tracing::debug!("Received binary message, not yet implemented");
            todo!()
        }
        Message::Ping(bytes) => {
            tracing::debug!("Received ping, sending pong");
            write
                .send(Message::Pong(bytes))
                .await
                .map_err(|e| e.into())?;
            Ok(MessageHandleResult::Continue)
        }
        Message::Pong(_bytes) => {
            tracing::debug!("Received pong");
            Ok(MessageHandleResult::Continue)
        }
        Message::Close(close_frame) => {
            tracing::debug!("Received close_frame from server");
            if let Some(close_frame) = close_frame {
                Err(anyhow::anyhow!(
                    "Connection closed by server with error: {}",
                    close_frame.reason.as_str()
                ))
            } else {
                Err(anyhow::anyhow!(
                    "Connection closed by server with unknown error"
                ))
            }
        }
        Message::Frame(_frame) => {
            tracing::debug!("Received frame, not yet implemented");
            todo!()
        }
    }
}

pub async fn wait_for_text_message_with_pings(
    read: &mut futures_util::stream::SplitStream<
        tokio_tungstenite::WebSocketStream<
            tokio_tungstenite::MaybeTlsStream<tokio::net::TcpStream>,
        >,
    >,
    write: &mut futures_util::stream::SplitSink<
        tokio_tungstenite::WebSocketStream<
            tokio_tungstenite::MaybeTlsStream<tokio::net::TcpStream>,
        >,
        Message,
    >,
    context: &str,
) -> anyhow::Result<String> {
    let mut ping_interval = tokio::time::interval(PING_INTERVAL);
    loop {
        tokio::select! {
            _ = ping_interval.tick() => {
                tracing::debug!("Sending periodic ping ({})", context);
                write.send(Message::Ping(vec![].into())).await?;
            }
            message = read.next() => {
                let Some(msg) = message else {
                    anyhow::bail!("Stream ended {}", context);
                };
                match handle_websocket_message(msg?, write).await? {
                    MessageHandleResult::TextBreak(text) => return Ok(text),
                    MessageHandleResult::Continue => {}
                }
            }
        }
    }
}

/// Connect to the hub WebSocket and perform initial handshake
pub async fn connect_and_handshake(
    ws_url: String,
    device_name: String,
) -> anyhow::Result<(
    ConnectDeviceWsEvent,
    tokio_tungstenite::WebSocketStream<tokio_tungstenite::MaybeTlsStream<tokio::net::TcpStream>>,
)> {
    tracing::debug!("Connecting to WebSocket: {}", ws_url);
    let (ws_stream, _) = connect_async(ws_url).await?;
    tracing::debug!("WebSocket connected, splitting streams");
    let (mut write, mut read) = ws_stream.split();

    let command = ConnectDeviceWsCommand {
        name: Some(device_name.clone()),
        sdp_answer: None,
    };
    let command_str = serde_json::to_string(&command)?;
    tracing::debug!("Sending initial command: {}", device_name);

    write.send(Message::Text(command_str.into())).await?;
    tracing::debug!("Initial command sent, waiting for event");

    let event_str =
        wait_for_text_message_with_pings(&mut read, &mut write, "during initial connection")
            .await?;
    let event: ConnectDeviceWsEvent = serde_json::from_str(event_str.as_str())?;
    tracing::debug!("Parsed ConnectDeviceWsEvent");

    // Reconstruct the stream from split halves
    use futures_util::stream::StreamExt;
    let ws_stream = read
        .reunite(write)
        .map_err(|e| anyhow::anyhow!("Failed to reunite stream: {}", e))?;
    Ok((event, ws_stream))
}
