use webrogue_hub_client::{
    debug_message_sender::send_debug_message,
    debug_messages::{DebugEvent, DebugIncomingMessageBody, GDBDataDebugEvent},
};
use webrogue_wasmtime::debugger::connection::PacketSender;
use webrtc::data_channel::DataChannel;

pub struct WebRTCPacketSender {
    pub data_channel: std::sync::Weak<dyn DataChannel>,
}

#[async_trait::async_trait]
impl PacketSender for WebRTCPacketSender {
    async fn send(&mut self, data: &[u8]) -> anyhow::Result<()> {
        let Some(data_channel) = self.data_channel.upgrade() else {
            anyhow::bail!("data_channel_weak.upgrade() failed");
        };
        let message = DebugIncomingMessageBody::Event(DebugEvent::GDBData(GDBDataDebugEvent {
            data: data.to_vec(),
        }));
        send_debug_message(data_channel.as_ref(), &message.to_bytes()?).await?;
        Ok(())
    }
}
