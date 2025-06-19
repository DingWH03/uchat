use serde::Deserialize;
use crate::protocol::RoleType;

#[derive(Deserialize, Debug)]
pub struct ChangeRoleRequest {
    pub user_id: u32,
    pub new_role: RoleType, 
}

#[derive(Deserialize, Debug)]
pub struct CheckUserDetailRequest {
    pub user_id: u32,
}

#[derive(Deserialize, Debug)]
pub struct DeleteUserRequest {
    pub user_id: u32,
}

#[derive(Deserialize, Debug)]
pub struct DeleteFriendshipRequest {
    pub user_id: u32,
    pub friend_id: u32,
}

#[derive(Deserialize, Debug)]
pub struct GetFriendsRequest {
    pub user_id: u32,
}