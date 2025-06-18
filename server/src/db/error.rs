use thiserror::Error;

#[derive(Debug, Error)]
pub enum DBError {
    #[error("SQLx 错误: {0}")]
    Sqlx(#[from] sqlx::Error),

    // 如果以后支持其他数据库，比如 diesel:
    // #[error("Diesel 错误: {0}")]
    // Diesel(#[from] diesel::result::Error),

    #[error("数据不存在")]
    NotFound,

    // #[error("违反约束条件: {0}")]
    // ConstraintViolation(String),

    // #[error("未知数据库错误: {0}")]
    // Unknown(String),
}
