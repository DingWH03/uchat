use thiserror::Error;
use bcrypt::BcryptError;

use crate::db::error::DBError;

#[derive(Debug, Error)]
pub enum RequestError {
    #[error("数据库错误: {0}")]
    Database(#[from] DBError),
    
    #[error("用户不存在")]
    UserNotFound,

    #[error("密码错误")]
    InvalidPassword,

    #[error("哈希错误: {0}")]
    Bcrypt(#[from] BcryptError),

    #[error("Session找不到")]
    SessionNotFound,

    #[error("Json序列化失败")]
    JsonError(#[from] serde_json::Error),
}

#[derive(Debug, Error)]
pub enum ManagerError {
    #[error("Database error: {0}")]
    DBError(#[from] DBError),

    #[error("Unauthorized")]
    Unauthorized,

    #[error("User not found")]
    UserNotFound,
}
