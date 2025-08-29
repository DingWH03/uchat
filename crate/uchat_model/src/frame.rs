#[derive(Debug, Copy, Clone)]
#[repr(u8)]
pub enum PayloadType {
    Json = 0,
    FileChunk = 1,
    VideoFrame = 2,
    AudioFrame = 3,
    Heartbeat = 4,
    // etc.
}

#[derive(Debug)]
pub struct Frame {
    pub payload_type: PayloadType,
    pub payload: Vec<u8>,
}

impl Frame {
    pub fn to_bytes(&self) -> Vec<u8> {
        let mut bytes = Vec::new();
        bytes.extend_from_slice(&[0xAB, 0xCD]); // magic
        bytes.push(self.payload_type as u8); // type
        bytes.extend_from_slice(&(self.payload.len() as u32).to_be_bytes()); // length
        bytes.extend_from_slice(&self.payload); // payload
        bytes
    }

    pub fn from_bytes(data: &[u8]) -> Option<Frame> {
        if data.len() < 7 || data[0] != 0xAB || data[1] != 0xCD {
            return None;
        }
        let payload_type = match data[2] {
            0 => PayloadType::Json,
            1 => PayloadType::FileChunk,
            2 => PayloadType::VideoFrame,
            3 => PayloadType::AudioFrame,
            4 => PayloadType::Heartbeat,
            _ => return None,
        };
        let length = u32::from_be_bytes([data[3], data[4], data[5], data[6]]) as usize;
        if data.len() < 7 + length {
            return None;
        }
        let payload = data[7..7 + length].to_vec();
        Some(Frame {
            payload_type,
            payload,
        })
    }
}
