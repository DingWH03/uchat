use std::sync::Arc;

#[cfg(feature = "mysql")]
use crate::db::mysql::MysqlDB;
#[cfg(feature = "postgres")]
use crate::db::postgresql::PgSqlDB;
use crate::db::{DB, InitDB};
use anyhow::{Ok, Result, anyhow};

pub enum DbType {
    #[cfg(feature = "mysql")]
    MySQL,
    #[cfg(feature = "postgres")]
    Postgres,
}

impl std::str::FromStr for DbType {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self> {
        match s.to_lowercase().as_str() {
            #[cfg(feature = "mysql")]
            "mysql" => Ok(DbType::MySQL),
            #[cfg(feature = "postgres")]
            "postgres" => Ok(DbType::Postgres),
            // "sqlite" => Ok(DbType::SQLite),
            #[allow(unreachable_patterns)]
            other => Err(anyhow!("不支持的数据库类型: {}", other)),
        }
    }
}

pub async fn create_database(db_type: DbType, database_url: &str) -> Result<Arc<dyn DB>> {
    match db_type {
        #[cfg(feature = "mysql")]
        DbType::MySQL => {
            let db = MysqlDB::init(database_url).await?;
            Ok(Arc::new(db))
        }
        #[cfg(feature = "postgres")]
        DbType::Postgres => {
            let db = PgSqlDB::init(database_url).await?;
            Ok(Arc::new(db))
        } // DbType::MariaDB => {
          //     let db = MariaDbDatabase::init(database_url).await?;
          //     Ok(Box::new(db))
          // }
    }
}
