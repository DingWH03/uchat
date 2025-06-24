use async_trait::async_trait;
use log::info;
mod minio;

use std::sync::Arc;

pub struct StorageConfig {
    pub endpoint: String,
    pub access_key: String,
    pub secret_key: String,
    pub bucket: String,
    pub base_url: String,
    pub local_dir: String,
}

pub enum StorageBackend {
    Minio,
    // Local,
}

pub async fn init_storage(backend: StorageBackend, config: &StorageConfig) -> Arc<dyn ObjectStorage> {
    match backend {
        StorageBackend::Minio => {
            let storage = minio::MinioStorage::new(
                &config.endpoint,
                &config.access_key,
                &config.secret_key,
                &config.bucket,
                &config.base_url,
            )
            .await
            .expect("Failed to init MinIO");
            storage.test_connection().await.expect("Failed to connect to MinIO");
            info!("MinIO storage initialized successfully");
            Arc::new(storage)
        }

        // StorageBackend::Local => {
        //     let storage = local::LocalStorage::new(&config.local_dir, &config.base_url);
        //     Arc::new(storage)
        // }
    }
}


#[async_trait]
pub trait ObjectStorage: Send + Sync {
    /// 上传文件，返回公开 URL
    async fn upload(
        &self,
        object_path: &str,
        data: &[u8],
        content_type: &str,
    ) -> anyhow::Result<String>;

    /// 删除文件
    async fn delete(&self, object_path: &str) -> anyhow::Result<()>;

    /// 获取公开 URL（可用于前端展示）
    fn get_url(&self, object_path: &str) -> String;
}
