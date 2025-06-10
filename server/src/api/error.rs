use thiserror::Error;
use bcrypt::BcryptError;
use sqlx::Error as SqlxError;

#[derive(Debug, Error)]
pub enum UserError {
    #[error("用户不存在")]
    UserNotFound,

    #[error("密码错误")]
    InvalidPassword,

    #[error("数据库错误: {0}")]
    Database(#[from] SqlxError),

    #[error("哈希错误: {0}")]
    Bcrypt(#[from] BcryptError),

    #[error("Session找不到")]
    SessionNotFound,
}

#[derive(Debug, Error)]
pub enum RequestError {
    #[error("用户信息错误")]
    UserError(#[from] UserError),
}