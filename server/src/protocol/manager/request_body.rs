use serde::Deserialize;
use crate::protocol::RoleType;

#[derive(Deserialize, Debug)]
pub struct ChangeRoleRequest {
    pub user_id: u32,
    pub new_role: RoleType, // 使用字符串，便于前端传参
}
