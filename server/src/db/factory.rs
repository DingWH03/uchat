use std::sync::Arc;

use anyhow::{Result, anyhow};
use crate::db::{InitDB, DB};
use crate::db::mysql::MysqlDB;

pub enum DbType {
    MySQL,
}

impl std::str::FromStr for DbType {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self> {
        match s.to_lowercase().as_str() {
            "mysql" => Ok(DbType::MySQL),
            // "postgres" => Ok(DbType::Postgres),
            // "sqlite" => Ok(DbType::SQLite),
            other => Err(anyhow!("不支持的数据库类型: {}", other)),
        }
    }
}

pub async fn create_database(db_type: DbType, database_url: &str) -> Result<Arc<dyn DB>> {
    match db_type {
        DbType::MySQL => {
            let db = MysqlDB::init(database_url).await?;
            Ok(Arc::new(db))
        }
        // DbType::MariaDB => {
        //     let db = MariaDbDatabase::init(database_url).await?;
        //     Ok(Box::new(db))
        // }
    }
}
