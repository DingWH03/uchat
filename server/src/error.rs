use crate::api::error::UserError;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum ServerError {
    #[error("用户模块错误: {0}")]
    UserError(#[from] UserError),
}