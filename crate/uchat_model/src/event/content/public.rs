use std::net::{IpAddr, Ipv6Addr};
use serde::{Serialize, Deserialize};
use byteorder::{BigEndian, ReadBytesExt, WriteBytesExt};
use std::io::{Cursor, Read};

use crate::{
    event::{content::{private::{LoginStatus, ProfileInfo}, EventContent}, ActorKind, Event}, frame::FrameError, model::{GroupId, Timestamp, UserId}, EventId
};

#[derive(Clone, Debug, Serialize, Deserialize, Eq, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum PublicLoginStatus { Success, Failed }

#[derive(Clone, Debug, Serialize, Deserialize, Eq, PartialEq)]
#[serde(tag = "type", content = "data", rename_all = "snake_case")]
pub enum PublicEventContent {
    LoginIn {
        status: PublicLoginStatus,
        ip_prefix: String, // e.g. "203.0.113.0/24"
    },
    LoginOut,
    UpdateProfile {
        profile_info: ProfileInfo,
    },
    // 空事件，代表错误或不可见事件
    NoContent,
}

#[derive(Clone, Debug, Serialize, Deserialize, Eq, PartialEq)]
pub struct PublicEvent {
    pub event_id: EventId,
    pub timestamp: Timestamp,
    pub actor: ActorKind,
    pub event_name: PublicEventContent,
}

/* ---------- PublicEvent 的二进制编解码（内聚在此） ----------

编码布局（BigEndian）：

PublicEvent:
  [ event_id(u64) | timestamp(i64) | actor(u8) | content_len(u32) | content_bytes... ]

PublicEventContent（content_bytes）：
  tag(u8) + fields...
  tag 映射：
    0 = LoginIn { status(u8), ip_prefix(string) }
    1 = LoginOut
    2 = UpdateProfile { user_id: Option<u32>, group_id: Option<u32> }
    3 = NoContent
*/

impl PublicEvent {
    #[inline]
    pub fn to_bytes(&self) -> Vec<u8> {
        let mut out = Vec::with_capacity(64);
        out.write_u64::<BigEndian>(self.event_id as u64).unwrap();
        out.write_i64::<BigEndian>(self.timestamp as i64).unwrap();
        out.write_u8(actor_kind_to_u8(&self.actor)).unwrap();

        let mut content_buf = Vec::with_capacity(32);
        encode_public_event_content(&mut content_buf, &self.event_name);

        out.write_u32::<BigEndian>(content_buf.len() as u32).unwrap();
        out.extend_from_slice(&content_buf);
        out
    }

    #[inline]
    pub fn from_bytes(payload: &[u8]) -> Result<Self, FrameError> {
        let mut c = Cursor::new(payload);
        let event_id = c.read_u64::<BigEndian>()? as EventId;
        let timestamp = c.read_i64::<BigEndian>()? as Timestamp;
        let actor = actor_kind_from_u8(c.read_u8()?)?;
        let clen = c.read_u32::<BigEndian>()? as usize;

        if c.get_ref().len() < c.position() as usize + clen {
            return Err(FrameError::Truncated);
        }
        let start = c.position() as usize;
        let content = decode_public_event_content(Cursor::new(&payload[start..start + clen]))?;

        Ok(PublicEvent { event_id, timestamp, actor, event_name: content })
    }
}

/* ---------- From<&Event> -> PublicEvent（保持不变） ---------- */

impl From<&Event> for PublicEvent {
    fn from(e: &Event) -> Self {
        let event_name = match &e.content {
            Some(EventContent::LoginIn(info)) => {
                let status = match info.status {
                    LoginStatus::Success => PublicLoginStatus::Success,
                    _ => PublicLoginStatus::Failed,
                };
                let ip_prefix = mask_ip_prefix(info.ip);
                PublicEventContent::LoginIn { status, ip_prefix }
            }
            // LoginFailed 事件对 public 不可见
            Some(EventContent::LoginFailed(_)) => PublicEventContent::NoContent,
            Some(EventContent::LoginOut) => {
                PublicEventContent::LoginOut
            }
            Some(EventContent::UpdateProfile(info)) => {
                PublicEventContent::UpdateProfile { profile_info: info.clone() }
            }
            None => panic!("No content found for event"),
        };

        PublicEvent {
            event_id: e.event_id,
            timestamp: e.timestamp,
            actor: e.actor_kind.clone(),
            event_name,
        }
    }
}

// ---------- 小工具：IP 前缀去敏 ----------

/// 对 IPv4 保留 /24 前缀；对 IPv6 保留 /64 前缀。
fn mask_ip_prefix(ip: IpAddr) -> String {
    match ip {
        IpAddr::V4(v4) => {
            let [a, b, c, _d] = v4.octets();
            format!("{}.{}.{}.0/24", a, b, c)
        }
        IpAddr::V6(v6) => {
            let seg = v6.segments();
            // 保留前 4 段，归零后缀，展示为 /64
            let masked = Ipv6Addr::new(seg[0], seg[1], seg[2], seg[3], 0, 0, 0, 0);
            format!("{}/64", masked)
        }
    }
}

/* ---------- 内部编解码辅助 ---------- */

#[inline]
fn write_string(out: &mut Vec<u8>, s: &str) {
    let b = s.as_bytes();
    out.write_u32::<BigEndian>(b.len() as u32).unwrap();
    out.extend_from_slice(b);
}
#[inline]
fn read_string(c: &mut Cursor<&[u8]>) -> Result<String, FrameError> {
    let len = c.read_u32::<BigEndian>()? as usize;
    let mut buf = vec![0u8; len];
    c.read_exact(&mut buf)?;
    Ok(String::from_utf8(buf)?)
}

#[inline]
fn write_opt_u32(out: &mut Vec<u8>, v: Option<u32>) {
    match v {
        Some(x) => { out.write_u8(1).unwrap(); out.write_u32::<BigEndian>(x).unwrap(); }
        None => { out.write_u8(0).unwrap(); }
    }
}
#[inline]
fn read_opt_u32(c: &mut Cursor<&[u8]>) -> Result<Option<u32>, FrameError> {
    let tag = c.read_u8()?;
    Ok(if tag == 1 { Some(c.read_u32::<BigEndian>()?) } else { None })
}

#[inline]
fn actor_kind_to_u8(a: &ActorKind) -> u8 {
    match a { ActorKind::System => 0, ActorKind::User => 1, ActorKind::Group => 2 }
}
#[inline]
fn actor_kind_from_u8(x: u8) -> Result<ActorKind, FrameError> {
    match x { 0 => Ok(ActorKind::System), 1 => Ok(ActorKind::User), 2 => Ok(ActorKind::Group), _ => Err(FrameError::InvalidKind(x)) }
}

fn encode_public_event_content(out: &mut Vec<u8>, ev: &PublicEventContent) {
    match ev {
        PublicEventContent::LoginIn { status, ip_prefix } => {
            out.write_u8(0).unwrap();
            out.write_u8(match status { PublicLoginStatus::Success => 0, PublicLoginStatus::Failed => 1 }).unwrap();
            write_string(out, ip_prefix);
        }
        PublicEventContent::LoginOut => {
            out.write_u8(1).unwrap();
        }
        PublicEventContent::UpdateProfile { profile_info } => {
            out.write_u8(2).unwrap();
            // 假定 UserId/GroupId 底层为 u32；若是别的整数类型请替换
            write_opt_u32(out, profile_info.user_id.map(|v| v as u32));
            write_opt_u32(out, profile_info.group_id.map(|v| v as u32));
        }
        PublicEventContent::NoContent => {
            out.write_u8(3).unwrap();
        }
    }
}

fn decode_public_event_content(mut c: Cursor<&[u8]>) -> Result<PublicEventContent, FrameError> {
    let tag = c.read_u8()?;
    match tag {
        0 => {
            let status = match c.read_u8()? {
                0 => PublicLoginStatus::Success,
                1 => PublicLoginStatus::Failed,
                x => return Err(FrameError::InvalidKind(x)),
            };
            let ip_prefix = read_string(&mut c)?;
            Ok(PublicEventContent::LoginIn { status, ip_prefix })
        }
        1 => Ok(PublicEventContent::LoginOut),
        2 => {
            let user_id = read_opt_u32(&mut c)?.map(|v| v as UserId);
            let group_id = read_opt_u32(&mut c)?.map(|v| v as GroupId);
            Ok(PublicEventContent::UpdateProfile { profile_info: ProfileInfo { user_id, group_id } })
        }
        3 => Ok(PublicEventContent::NoContent),
        x => Err(FrameError::InvalidKind(x)),
    }
}
