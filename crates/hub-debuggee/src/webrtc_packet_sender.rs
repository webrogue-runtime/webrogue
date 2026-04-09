use webrogue_debugger::PacketSender;
use webrogue_hub_client::debug_messages::{DebugEvent, DebugIncomingMessage, GDBDataDebugEvent};
use webrtc::data_channel::RTCDataChannel;

pub struct WebRTCPacketSender {
    pub data_channel: std::sync::Weak<RTCDataChannel>,
}

#[async_trait::async_trait]
impl PacketSender for WebRTCPacketSender {
    async fn send(&mut self, data: &[u8]) -> anyhow::Result<()> {
        let Some(data_channel) = self.data_channel.upgrade() else {
            return Ok(());
        };
        let message = DebugIncomingMessage::Event(DebugEvent::GDBData(GDBDataDebugEvent {
            data: data.to_vec(),
        }));
        data_channel.send(&message.to_bytes()?.into()).await?;
        Ok(())
    }
}
