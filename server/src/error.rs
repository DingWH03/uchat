use crate::api::error::{ManagerError, RequestError};
use thiserror::Error;

#[derive(Debug, Error)]
pub enum ServerError {
    #[error("api请求错误: {0}")]
    RequestError(#[from] RequestError),
    #[error("manager请求错误: {0}")]
    ManagerError(#[from] ManagerError)
}
