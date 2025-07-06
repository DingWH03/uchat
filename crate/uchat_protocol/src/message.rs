use serde::{Deserialize, Serialize};

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
