use serde::{Deserialize, Serialize};

#[derive(Deserialize, Debug)]
#[serde(tag = "type")]
pub enum ClientMessage {
    SendMessage {
        receiver: u32,
        message: String,
    },
    SendGroupMessage {
        group_id: u32,
        message: String,
    },
}

#[derive(Serialize, Debug)]
#[serde(tag = "type")]
pub enum ServerMessage {
    SendMessage {
        sender: u32,
        message: String,
    },
    SendGroupMessage {
        sender: u32,
        group_id: u32,
        message: String,
    },
    OnlineMessage {
        friend_id: u32
    },
    OfflineMessage {
        friend_id: u32
    }
}