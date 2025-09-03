use serde::{Deserialize, Serialize};
use byteorder::{BigEndian, ReadBytesExt, WriteBytesExt};
use std::io::{Cursor, Read};

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
    OnlineMessage {
        friend_id: u32,
    },
    OfflineMessage {
        friend_id: u32,
    },
}

#[repr(u8)]
enum Kind {
    SendMessage = 0,
    SendGroupMessage = 1,
    OnlineMessage = 2,
    OfflineMessage = 3,
}

#[derive(Debug)]
pub enum DecodeError {
    UnexpectedEof,
    InvalidDiscriminant(u8),
    Utf8(std::string::FromUtf8Error),
    Io(std::io::Error),
}
impl From<std::io::Error> for DecodeError { fn from(e: std::io::Error) -> Self { Self::Io(e) } }
impl From<std::string::FromUtf8Error> for DecodeError { fn from(e: std::string::FromUtf8Error) -> Self { Self::Utf8(e) } }

impl ServerMessage {
    /// 新增：高效二进制编码方法（用于 WebSocket 二进制帧）
    pub fn to_bytes(&self) -> Vec<u8> {
        let mut out = Vec::with_capacity(64);
        match self {
            ServerMessage::SendMessage { message_id, sender, receiver, message, timestamp } => {
                out.push(Kind::SendMessage as u8);
                out.write_u64::<BigEndian>(*message_id).unwrap();
                out.write_u32::<BigEndian>(*sender).unwrap();
                out.write_u32::<BigEndian>(*receiver).unwrap();
                out.write_i64::<BigEndian>(*timestamp).unwrap();
                let m = message.as_bytes();
                out.write_u32::<BigEndian>(m.len() as u32).unwrap();
                out.extend_from_slice(m);
            }
            ServerMessage::SendGroupMessage { message_id, sender, group_id, message, timestamp } => {
                out.push(Kind::SendGroupMessage as u8);
                out.write_u64::<BigEndian>(*message_id).unwrap();
                out.write_u32::<BigEndian>(*sender).unwrap();
                out.write_u32::<BigEndian>(*group_id).unwrap();
                out.write_i64::<BigEndian>(*timestamp).unwrap();
                let m = message.as_bytes();
                out.write_u32::<BigEndian>(m.len() as u32).unwrap();
                out.extend_from_slice(m);
            }
            ServerMessage::OnlineMessage { friend_id } => {
                out.push(Kind::OnlineMessage as u8);
                out.write_u32::<BigEndian>(*friend_id).unwrap();
            }
            ServerMessage::OfflineMessage { friend_id } => {
                out.push(Kind::OfflineMessage as u8);
                out.write_u32::<BigEndian>(*friend_id).unwrap();
            }
        }
        out
    }

    /// 可选：解码，便于测试/服务内部环路使用
    pub fn from_bytes(bytes: &[u8]) -> Result<Self, DecodeError> {
        let mut c = Cursor::new(bytes);
        let tag = c.read_u8()?;
        match tag {
            x if x == Kind::SendMessage as u8 => {
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
            x if x == Kind::SendGroupMessage as u8 => {
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
            x if x == Kind::OnlineMessage as u8 => {
                let friend_id = c.read_u32::<BigEndian>()?;
                Ok(ServerMessage::OnlineMessage { friend_id })
            }
            x if x == Kind::OfflineMessage as u8 => {
                let friend_id = c.read_u32::<BigEndian>()?;
                Ok(ServerMessage::OfflineMessage { friend_id })
            }
            other => Err(DecodeError::InvalidDiscriminant(other)),
        }
    }
}