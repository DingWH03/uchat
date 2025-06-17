use crate::{api::error::RequestError};
use thiserror::Error;

#[derive(Debug, Error)]
pub enum ServerError {
    #[error("api请求错误: {0}")]
    RequestError(#[from] RequestError),
}
