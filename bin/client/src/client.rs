use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug)]
#[serde(tag = "action")]
pub enum ClientRequest {
    #[serde(rename = "request")]
    Request { request: String },
    #[serde(rename = "objrequest")]
    ObjRequest { request: String, id: u32 },
    #[serde(rename = "namerequest")]
    NameRequest { request: String, name: String },
    #[serde(rename = "messagesrequest")]
    MessagesRequest { group: bool, id: u32, offset: u32 },
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(tag = "type")]
pub enum ClientMessage {
    SendMessage { receiver: u32, message: String },
    SendGroupMessage { group_id: u32, message: String },
}
