pub mod private;
pub mod public;

use serde::{Serialize, Deserialize};
use crate::event::{content::{private::{LoginInfo, ProfileInfo}, public::PublicEvent}, Event};

#[derive(Clone, Debug, Serialize, Deserialize, Eq, PartialEq)]
#[serde(tag = "type", content = "data", rename_all = "snake_case")]
pub enum EventContent {
    LoginIn(LoginInfo),
    LoginOut,
    LoginFailed(LoginInfo),
    UpdateProfile(ProfileInfo),
}

impl Event {
    pub fn to_public(&self) -> PublicEvent {
        PublicEvent::from(self)
    }
}
