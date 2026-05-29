use crate::debug_messages::DebugMessage;

pub struct DebugMessageReceiver {
    fragment_buffer: Vec<u8>,
}

impl DebugMessageReceiver {
    pub fn new() -> Self {
        Self {
            fragment_buffer: Vec::new(),
        }
    }

    pub fn receive(&mut self, data: &[u8]) -> anyhow::Result<Option<Vec<u8>>> {
        let mut message = DebugMessage::from_bytes(data)?;
        anyhow::ensure!(message.version == crate::debug_messages::VERSION);
        self.fragment_buffer.append(&mut message.fragment);
        if !message.is_last_fragment {
            return Ok(None);
        }
        Ok(Some(std::mem::replace(
            &mut self.fragment_buffer,
            Vec::new(),
        )))
    }
}
