use webrtc::data_channel::RTCDataChannel;

use crate::debug_messages::DebugMessage;

pub async fn send_debug_message(data_channel: &RTCDataChannel, data: &[u8]) -> anyhow::Result<()> {
    // TODO increase chunk size
    let fragments = data.chunks(100).collect::<Vec<_>>();
    for (i, fragment) in fragments.iter().enumerate() {
        let message = DebugMessage {
            version: crate::debug_messages::VERSION,
            fragment: fragment.to_vec(),
            is_last_fragment: i == fragments.len() - 1,
        };
        data_channel.send(&message.to_bytes()?.into()).await?;
    }
    Ok(())
}
