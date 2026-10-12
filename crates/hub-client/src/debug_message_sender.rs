use webrtc::data_channel::DataChannel;

use crate::debug_messages::DebugMessage;

pub async fn send_debug_message(data_channel: &dyn DataChannel, data: &[u8]) -> anyhow::Result<()> {
    // TODO increase chunk size
    let fragments = data.chunks(1100).collect::<Vec<_>>();
    for (i, fragment) in fragments.iter().enumerate() {
        let message = DebugMessage {
            version: crate::debug_messages::VERSION,
            fragment: fragment.to_vec(),
            is_last_fragment: i == fragments.len() - 1,
        };
        let message_bytes = message.to_bytes()?;
        data_channel.send(message_bytes.as_slice().into()).await?;
    }
    Ok(())
}
