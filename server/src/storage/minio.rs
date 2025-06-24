use anyhow::{Context, Result};
use async_trait::async_trait;
use aws_sdk_s3::config::Builder as S3ConfigBuilder;
use aws_sdk_s3::config::Credentials;
use aws_sdk_s3::primitives::ByteStream;
use aws_sdk_s3::types::Bucket;
use aws_sdk_s3::{Client, config::Region};
use log::info;

use crate::storage::ObjectStorage;

#[derive(Clone)]
pub struct MinioStorage {
    client: Client,
    bucket: String,
    base_url: String,
}

impl MinioStorage {
    pub async fn new(
        endpoint: &str,
        access_key: &str,
        secret_key: &str,
        bucket: &str,
        base_url: &str,
    ) -> Result<Self> {
        let config = aws_config::defaults(aws_config::BehaviorVersion::latest())
            .endpoint_url(endpoint)
            .region(Region::new("custom"))
            .credentials_provider(Credentials::new(
                access_key, secret_key, None, None, "static",
            ))
            .load()
            .await;
        let s3_config = S3ConfigBuilder::from(&config)
            .force_path_style(true) // 不构建虚拟主机样式的 URL
            .build();

        let client = Client::from_conf(s3_config);
        // let client = Client::new(&s3_config);

        Ok(Self {
            client,
            bucket: bucket.to_string(),
            base_url: base_url.trim_end_matches('/').to_string(),
        })
    }
}

#[async_trait]
impl ObjectStorage for MinioStorage {
    async fn upload(&self, object_path: &str, data: &[u8], content_type: &str) -> Result<String> {
        let body = ByteStream::from(data.to_vec());

        self.client
            .put_object()
            .bucket(&self.bucket)
            .key(object_path)
            .body(body)
            .content_type(content_type)
            .send()
            .await
            .context("failed to upload to MinIO (S3)")?;

        Ok(self.get_url(object_path))
    }

    async fn delete(&self, object_path: &str) -> Result<()> {
        self.client
            .delete_object()
            .bucket(&self.bucket)
            .key(object_path)
            .send()
            .await
            .context("failed to delete from MinIO (S3)")?;
        Ok(())
    }

    fn get_url(&self, object_path: &str) -> String {
        format!("{}/{}", self.base_url, object_path)
    }

    /// 删除指定 prefix 下除 except_keys 之外的对象
    async fn delete_prefix_except(&self, prefix: &str, except_keys: &[&str]) -> anyhow::Result<()> {
        // 列举指定前缀下所有对象
        let resp = self.client
            .list_objects_v2()
            .bucket(&self.bucket)
            .prefix(prefix)
            .send()
            .await
            .context("failed to list objects in MinIO (S3)")?;

        if let Some(contents) = resp.contents {
            for obj in contents {
                if let Some(key) = obj.key {
                    if !except_keys.contains(&key.as_str()) {
                        self.delete(&key).await?;
                    }
                }
            }
        }

        Ok(())
    }
}

impl MinioStorage {
    /// 测试与 MinIO 服务器的连接是否正常
    pub async fn test_connection(&self) -> Result<()> {
        let resp = self.client.list_buckets().send().await?;
        let mut exists = false;

        if let Some(buckets) = &resp.buckets {
            info!("MinIO连接成功，当前Buckets列表:");
            for bucket in buckets {
                if let Some(name) = &bucket.name {
                    info!(" - {}", name);
                    if name == &self.bucket {
                        exists = true;
                    }
                }
            }
        } else {
            info!("MinIO连接成功，但未获取到任何桶信息");
        }

        if !exists {
            info!("Bucket '{}' 不存在，正在自动创建...", self.bucket);
            self.client
                .create_bucket()
                .bucket(&self.bucket)
                .send()
                .await?;
            info!("Bucket '{}' 创建成功", self.bucket);
        }

        // 检查现有策略是否已包含公共读取权限
        let need_update_policy = match self
            .client
            .get_bucket_policy()
            .bucket(&self.bucket)
            .send()
            .await
        {
            Ok(resp) => {
                let current_policy = resp.policy.unwrap_or_default();
                !current_policy.contains("\"s3:GetObject\"") // 可再加更严格判断
            }
            Err(_) => {
                // 获取失败，说明无策略或权限不足 -> 需要设置
                true
            }
        };

        if need_update_policy {
            info!("正在设置 Bucket '{}' 的公共读取策略...", self.bucket);
            let policy = serde_json::json!({
                "Version": "2012-10-17",
                "Statement": [{
                    "Sid": "PublicReadGetObject",
                    "Effect": "Allow",
                    "Principal": "*",
                    "Action": ["s3:GetObject"],
                    "Resource": format!("arn:aws:s3:::{}/*", self.bucket)
                }]
            });

            self.client
                .put_bucket_policy()
                .bucket(&self.bucket)
                .policy(policy.to_string())
                .send()
                .await?;

            info!("公共读取策略设置完成");
        } else {
            info!("Bucket '{}' 已存在且策略正确，跳过设置", self.bucket);
        }

        Ok(())
    }
}
