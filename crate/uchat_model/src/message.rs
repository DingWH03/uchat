use byteorder::{BigEndian, ReadBytesExt, WriteBytesExt};
use std::io::{Cursor, Read};
use serde::{Deserialize, Serialize};
use crate::{
    frame::{FrameCodec, FrameError, Direction},
    event::content::public::PublicEvent,
};

#[derive(Deserialize, Debug)]
#[serde(tag = "type")]
pub enum ClientMessage {
    SendMessage { receiver: u32, message: String },
    SendGroupMessage { group_id: u32, message: String },
}

#[derive(Serialize, Debug)]
#[serde(tag = "type")]
pub enum ServerMessage {
    SendMessage {
        message_id: u64, // 消息ID
        sender: u32,
        receiver: u32,
        message: String,
        timestamp: i64, // 使用 i64 存储时间戳，单位为秒
    },
    SendGroupMessage {
        message_id: u64, // 消息ID
        sender: u32,
        group_id: u32,
        message: String,
        timestamp: i64, // 使用 i64 存储时间戳，单位为秒
    },
    Event(PublicEvent),
}

/* ---------------- ClientMessage: C2S ---------------- */

impl ClientMessage {
    #[inline]
    fn kind_u8(&self) -> u8 {
        match self {
            ClientMessage::SendMessage { .. } => 0,
            ClientMessage::SendGroupMessage { .. } => 1,
        }
    }

    fn encode_payload(&self, out: &mut Vec<u8>) {
        match self {
            ClientMessage::SendMessage { receiver, message } => {
                out.write_u32::<BigEndian>(*receiver).unwrap();
                let m = message.as_bytes();
                out.write_u32::<BigEndian>(m.len() as u32).unwrap();
                out.extend_from_slice(m);
            }
            ClientMessage::SendGroupMessage { group_id, message } => {
                out.write_u32::<BigEndian>(*group_id).unwrap();
                let m = message.as_bytes();
                out.write_u32::<BigEndian>(m.len() as u32).unwrap();
                out.extend_from_slice(m);
            }
        }
    }

    fn decode_payload(kind: u8, mut c: Cursor<&[u8]>) -> Result<Self, FrameError> {
        match kind {
            0 => {
                let receiver = c.read_u32::<BigEndian>()?;
                let len = c.read_u32::<BigEndian>()? as usize;
                let mut buf = vec![0u8; len];
                c.read_exact(&mut buf)?;
                let message = String::from_utf8(buf)?;
                Ok(ClientMessage::SendMessage { receiver, message })
            }
            1 => {
                let group_id = c.read_u32::<BigEndian>()?;
                let len = c.read_u32::<BigEndian>()? as usize;
                let mut buf = vec![0u8; len];
                c.read_exact(&mut buf)?;
                let message = String::from_utf8(buf)?;
                Ok(ClientMessage::SendGroupMessage { group_id, message })
            }
            x => Err(FrameError::InvalidKind(x)),
        }
    }
}

impl FrameCodec for ClientMessage {
    const DIR: Direction = Direction::C2S;
    fn kind(&self) -> u8 { self.kind_u8() }

    fn to_bytes(&self) -> Vec<u8> {
        let mut out = Vec::with_capacity(64);
        self.encode_payload(&mut out);
        out
    }

    fn from_bytes(kind: u8, payload: &[u8]) -> Result<Self, FrameError> {
        Self::decode_payload(kind, Cursor::new(payload))
    }
}

/* ---------------- ServerMessage: S2C ---------------- */

impl ServerMessage {
    #[inline]
    fn kind_u8(&self) -> u8 {
        match self {
            ServerMessage::SendMessage { .. } => 0,
            ServerMessage::SendGroupMessage { .. } => 1,
            ServerMessage::Event(_) => 2,
        }
    }

    fn encode_payload(&self, out: &mut Vec<u8>) {
        match self {
            ServerMessage::SendMessage { message_id, sender, receiver, message, timestamp } => {
                out.write_u64::<BigEndian>(*message_id).unwrap();
                out.write_u32::<BigEndian>(*sender).unwrap();
                out.write_u32::<BigEndian>(*receiver).unwrap();
                out.write_i64::<BigEndian>(*timestamp).unwrap();
                let m = message.as_bytes();
                out.write_u32::<BigEndian>(m.len() as u32).unwrap();
                out.extend_from_slice(m);
            }
            ServerMessage::SendGroupMessage { message_id, sender, group_id, message, timestamp } => {
                out.write_u64::<BigEndian>(*message_id).unwrap();
                out.write_u32::<BigEndian>(*sender).unwrap();
                out.write_u32::<BigEndian>(*group_id).unwrap();
                out.write_i64::<BigEndian>(*timestamp).unwrap();
                let m = message.as_bytes();
                out.write_u32::<BigEndian>(m.len() as u32).unwrap();
                out.extend_from_slice(m);
            }
            ServerMessage::Event(ev) => {
                let bytes = ev.to_bytes();
                out.extend_from_slice(&bytes);
            }
        }
    }

    fn decode_payload(kind: u8, mut c: Cursor<&[u8]>) -> Result<Self, FrameError> {
        match kind {
            0 => {
                let message_id = c.read_u64::<BigEndian>()?;
                let sender = c.read_u32::<BigEndian>()?;
                let receiver = c.read_u32::<BigEndian>()?;
                let timestamp = c.read_i64::<BigEndian>()?;
                let len = c.read_u32::<BigEndian>()? as usize;
                let mut buf = vec![0u8; len];
                c.read_exact(&mut buf)?;
                let message = String::from_utf8(buf)?;
                Ok(ServerMessage::SendMessage { message_id, sender, receiver, message, timestamp })
            }
            1 => {
                let message_id = c.read_u64::<BigEndian>()?;
                let sender = c.read_u32::<BigEndian>()?;
                let group_id = c.read_u32::<BigEndian>()?;
                let timestamp = c.read_i64::<BigEndian>()?;
                let len = c.read_u32::<BigEndian>()? as usize;
                let mut buf = vec![0u8; len];
                c.read_exact(&mut buf)?;
                let message = String::from_utf8(buf)?;
                Ok(ServerMessage::SendGroupMessage { message_id, sender, group_id, message, timestamp })
            }
            2 => {
                // 剩余 payload 全部属于 PublicEvent
                let remaining = {
                    let start = c.position() as usize;
                    &c.get_ref()[start..]
                };
                let ev = PublicEvent::from_bytes(remaining)?;
                Ok(ServerMessage::Event(ev))
            }
            x => Err(FrameError::InvalidKind(x)),
        }
    }
}

impl FrameCodec for ServerMessage {
    const DIR: Direction = Direction::S2C;
    fn kind(&self) -> u8 { self.kind_u8() }

    fn to_bytes(&self) -> Vec<u8> {
        let mut out = Vec::with_capacity(96);   // 预估容量
        self.encode_payload(&mut out);
        out
    }

    fn from_bytes(kind: u8, payload: &[u8]) -> Result<Self, FrameError> {
        Self::decode_payload(kind, Cursor::new(payload))
    }
}
