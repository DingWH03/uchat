//! event_models.rs
use std::net::{IpAddr, Ipv4Addr, Ipv6Addr};
use serde::{Serialize, Deserialize};
use std::fmt;

// ---------- 基础类型别名（按你的要求） ----------
pub type UserId = u32;
pub type GroupId = u32;
pub type MessageId = u64;
pub type Ver = u32;
pub type Timestamp = i64;

// ---------- 你的领域模型（便于阅读/业务友好：创建/读取都用它） ----------

#[derive(Clone, Debug, Serialize, Deserialize, Eq, PartialEq)]
pub enum Actor {
    System,
    User(UserId),
    Group(GroupId),
}

#[derive(Clone, Debug, Serialize, Deserialize, Eq, PartialEq)]
#[serde(tag = "kind", content = "ver", rename_all = "snake_case")]
pub enum UserAgent {
    Web(Ver),
    Mobile(Ver),
    Desktop(Ver),
    Other,
}

#[derive(Clone, Debug, Serialize, Deserialize, Eq, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum LoginStatus {
    Success,
    BadPassword,
    UserNotFound,
    Disabled,
}

#[derive(Clone, Debug, Serialize, Deserialize, Eq, PartialEq)]
pub struct LoginInfo {
    pub status: LoginStatus,
    pub user_id: UserId,
    pub ip: IpAddr,
    pub user_agent: UserAgent,
}

#[derive(Clone, Debug, Serialize, Deserialize, Eq, PartialEq)]
#[serde(tag = "type", content = "data", rename_all = "snake_case")]
pub enum EventData {
    LoginIn(LoginInfo),
}

#[derive(Clone, Debug, Serialize, Deserialize, Eq, PartialEq)]
pub struct Event {
    pub event_id: MessageId,
    pub timestamp: Timestamp, // 建议：使用秒级/毫秒级 Unix 时间戳，由外部保证时区
    pub actor: Actor,
    pub event_name: EventData,
}

// ---------- 平坦结构（便于存储：行式/列式/日志，弱耦合） ----------

#[derive(Clone, Debug, Serialize, Deserialize, Eq, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum ActorKind { System, User, Group }

#[derive(Clone, Debug, Serialize, Deserialize, Eq, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum EventKind { LoginIn }

#[derive(Clone, Debug, Serialize, Deserialize, Eq, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum UserAgentKind { Web, Mobile, Desktop, Other }

/// 注意：所有与具体事件相关的字段都做成 Option，便于水平扩展更多事件类型。
#[derive(Clone, Debug, Serialize, Deserialize, Eq, PartialEq)]
pub struct EventRow {
    pub event_id: MessageId,
    pub timestamp: Timestamp,

    pub actor_kind: ActorKind,
    pub actor_user_id: Option<UserId>,
    pub actor_group_id: Option<GroupId>,

    pub event_kind: EventKind,

    // --- LoginIn 专用扁平字段 ---
    pub login_status: Option<LoginStatus>,
    pub login_user_id: Option<UserId>,
    pub ip: Option<IpAddr>,
    pub user_agent_kind: Option<UserAgentKind>,
    pub user_agent_ver: Option<Ver>,
}

// ---------- 对外可见（去敏结构）：易生成、默认安全 ----------

#[derive(Clone, Debug, Serialize, Deserialize, Eq, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum PublicActor { System, User, Group }

#[derive(Clone, Debug, Serialize, Deserialize, Eq, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum PublicLoginStatus { Success, Failed }

#[derive(Clone, Debug, Serialize, Deserialize, Eq, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum PublicUserAgent { Web, Mobile, Desktop, Other }

#[derive(Clone, Debug, Serialize, Deserialize, Eq, PartialEq)]
#[serde(tag = "type", content = "data", rename_all = "snake_case")]
pub enum PublicEventData {
    /// 对用户不暴露内部 ID；IP 只展示前缀；登录失败不泄露具体原因。
    LoginIn {
        status: PublicLoginStatus,
        ip_prefix: String,         // 例： "203.0.113.0/24" 或 "2001:db8:85a3::/64"
        user_agent: PublicUserAgent,
    },
}

#[derive(Clone, Debug, Serialize, Deserialize, Eq, PartialEq)]
pub struct PublicEvent {
    pub event_id: MessageId,
    pub timestamp: Timestamp,
    pub actor: PublicActor,
    pub event_name: PublicEventData,
}

// ---------- 转换错误（不使用 thiserror） ----------

#[derive(Debug, Clone, Eq, PartialEq)]
pub enum ConvertError {
    MissingField(&'static str),
    InvalidCombination,
}

impl fmt::Display for ConvertError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ConvertError::MissingField(s) => write!(f, "missing field for event kind: {}", s),
            ConvertError::InvalidCombination => write!(f, "invalid combination of fields"),
        }
    }
}

impl std::error::Error for ConvertError {}

// ---------- Readable <-> Storage ----------

impl Event {
    /// 业务友好 -> 存储扁平
    pub fn to_storage(&self) -> EventRow {
        let (actor_kind, actor_user_id, actor_group_id) = match &self.actor {
            Actor::System => (ActorKind::System, None, None),
            Actor::User(uid) => (ActorKind::User, Some(*uid), None),
            Actor::Group(gid) => (ActorKind::Group, None, Some(*gid)),
        };

        let mut row = EventRow {
            event_id: self.event_id,
            timestamp: self.timestamp,
            actor_kind,
            actor_user_id,
            actor_group_id,
            event_kind: match self.event_name {
                EventData::LoginIn(_) => EventKind::LoginIn,
            },
            login_status: None,
            login_user_id: None,
            ip: None,
            user_agent_kind: None,
            user_agent_ver: None,
        };

        match &self.event_name {
            EventData::LoginIn(info) => {
                row.login_status = Some(info.status.clone());
                row.login_user_id = Some(info.user_id);
                row.ip = Some(info.ip);
                let (k, v) = split_user_agent(&info.user_agent);
                row.user_agent_kind = Some(k);
                row.user_agent_ver = v;
            }
        }

        row
    }

    /// 存储扁平 -> 业务友好（严格校验必要字段）
    pub fn from_storage(row: EventRow) -> Result<Self, ConvertError> {
        let actor = match row.actor_kind {
            ActorKind::System => Actor::System,
            ActorKind::User   => Actor::User(row.actor_user_id.ok_or(ConvertError::MissingField("actor_user_id"))?),
            ActorKind::Group  => Actor::Group(row.actor_group_id.ok_or(ConvertError::MissingField("actor_group_id"))?),
        };

        let event_name = match row.event_kind {
            EventKind::LoginIn => {
                let status = row.login_status.ok_or(ConvertError::MissingField("login_status"))?;
                let uid    = row.login_user_id.ok_or(ConvertError::MissingField("login_user_id"))?;
                let ip     = row.ip.ok_or(ConvertError::MissingField("ip"))?;
                let ua_k   = row.user_agent_kind.ok_or(ConvertError::MissingField("user_agent_kind"))?;
                let ua_v   = row.user_agent_ver;
                let ua     = merge_user_agent(ua_k, ua_v)?;
                EventData::LoginIn(LoginInfo { status, user_id: uid, ip, user_agent: ua })
            }
        };

        Ok(Event {
            event_id: row.event_id,
            timestamp: row.timestamp,
            actor,
            event_name,
        })
    }
}

// ---------- Readable -> Public（去敏） ----------

impl From<&Event> for PublicEvent {
    fn from(e: &Event) -> Self {
        let actor = match e.actor {
            Actor::System => PublicActor::System,
            Actor::User(_) => PublicActor::User,
            Actor::Group(_) => PublicActor::Group,
        };

        let event_name = match &e.event_name {
            EventData::LoginIn(info) => {
                let status = match info.status {
                    LoginStatus::Success => PublicLoginStatus::Success,
                    // 失败类统一收敛为 Failed，避免暴露存在性/口令策略
                    LoginStatus::BadPassword | LoginStatus::UserNotFound | LoginStatus::Disabled => PublicLoginStatus::Failed,
                };
                let ip_prefix = mask_ip_prefix(info.ip);
                let user_agent = match info.user_agent {
                    UserAgent::Web(_) => PublicUserAgent::Web,
                    UserAgent::Mobile(_) => PublicUserAgent::Mobile,
                    UserAgent::Desktop(_) => PublicUserAgent::Desktop,
                    UserAgent::Other => PublicUserAgent::Other,
                };

                PublicEventData::LoginIn { status, ip_prefix, user_agent }
            }
        };

        PublicEvent {
            event_id: e.event_id,
            timestamp: e.timestamp,
            actor,
            event_name,
        }
    }
}

// ---------- 小工具：UA 分解/合并、IP 前缀去敏 ----------

fn split_user_agent(ua: &UserAgent) -> (UserAgentKind, Option<Ver>) {
    match ua {
        UserAgent::Web(v)     => (UserAgentKind::Web,     Some(*v)),
        UserAgent::Mobile(v)  => (UserAgentKind::Mobile,  Some(*v)),
        UserAgent::Desktop(v) => (UserAgentKind::Desktop, Some(*v)),
        UserAgent::Other      => (UserAgentKind::Other,   None),
    }
}

fn merge_user_agent(kind: UserAgentKind, ver: Option<Ver>) -> Result<UserAgent, ConvertError> {
    Ok(match (kind, ver) {
        (UserAgentKind::Web,     Some(v)) => UserAgent::Web(v),
        (UserAgentKind::Mobile,  Some(v)) => UserAgent::Mobile(v),
        (UserAgentKind::Desktop, Some(v)) => UserAgent::Desktop(v),
        (UserAgentKind::Other,   _      ) => UserAgent::Other,
        // 如果缺版本但类型需要版本，则认为存储数据不完整
        _ => return Err(ConvertError::MissingField("user_agent_ver")),
    })
}

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

// ---------- 便捷 API ----------

impl Event {
    /// 直接产出 Public 去敏版本
    pub fn to_public(&self) -> PublicEvent {
        PublicEvent::from(self)
    }
}

// ---------- 示例（可选） ----------

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn round_trip_login_event() {
        // 构造可读事件
        let readable = Event {
            event_id: 42,
            timestamp: 1_725_000_000,
            actor: Actor::User(123),
            event_name: EventData::LoginIn(LoginInfo {
                status: LoginStatus::BadPassword,
                user_id: 123,
                ip: IpAddr::V4(Ipv4Addr::new(203, 0, 113, 7)),
                // Ver 是 u32：例如 1.2.3 可编码为 10203 或 0x010203
                user_agent: UserAgent::Web(10203),
            }),
        };

        // 可读 -> 存储
        let row = readable.to_storage();

        // 存储 -> 可读（严格还原）
        let readable2 = Event::from_storage(row).unwrap();
        assert_eq!(readable, readable2);

        // 去敏
        let public = readable.to_public();
        if let PublicEventData::LoginIn { status, ip_prefix, user_agent } = public.event_name {
            assert_eq!(status, PublicLoginStatus::Failed); // 失败类统一
            assert_eq!(ip_prefix, "203.0.113.0/24");
            assert_eq!(user_agent, PublicUserAgent::Web);
        } else {
            panic!("unexpected public event data");
        }
    }
}
