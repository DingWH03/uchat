use std::net::{IpAddr, Ipv6Addr};
use serde::{Serialize, Deserialize};
use crate::{event::{content::{private::{LoginStatus, ProfileInfo}, EventContent}, ActorKind, Event}, model::{MessageId, Timestamp}};


#[derive(Clone, Debug, Serialize, Deserialize, Eq, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum PublicLoginStatus { Success, Failed }

#[derive(Clone, Debug, Serialize, Deserialize, Eq, PartialEq)]
#[serde(tag = "type", content = "data", rename_all = "snake_case")]
pub enum PublicEventContent {
    LoginIn {
        status: PublicLoginStatus,
        ip_prefix: String, // Example: "203.0.113.0/24"
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
    pub event_id: MessageId,
    pub timestamp: Timestamp,
    pub actor: ActorKind,
    pub event_name: PublicEventContent,
}

// ---------- 转换为公共版本 ----------

impl From<&Event> for PublicEvent {
    fn from(e: &Event) -> Self {


        let event_name = match &e.content {
            Some(EventContent::LoginIn(info)) => {
                let status = match info.status {
                    LoginStatus::Success => PublicLoginStatus::Success,
                    // 失败类统一收敛为 Failed
                    _ => PublicLoginStatus::Failed,
                };
                let ip_prefix = mask_ip_prefix(info.ip);

                PublicEventContent::LoginIn { status, ip_prefix }
            }
            // LoginFailed时间理应对public不可见
            Some(EventContent::LoginFailed(_)) => {
                PublicEventContent::NoContent
            }
            Some(EventContent::LoginOut) => {
                // 对于登出事件，不包含 IP 信息
                PublicEventContent::LoginIn { status: PublicLoginStatus::Success, ip_prefix: "N/A".to_string() }
            }
            Some(EventContent::UpdateProfile(info)) => {
                PublicEventContent::UpdateProfile {
                    profile_info: info.clone(),
                }
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