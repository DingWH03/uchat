use std::sync::Arc;

// src/utils.rs
use anyhow::Result;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::sync::Mutex;
use crate::protocol::{ServerResponse, ClientRequest};

// 实现 writer_packet 函数
pub async fn writer_packet(writer: &mut Arc<Mutex<tokio::io::BufWriter<tokio::net::tcp::OwnedWriteHalf>>>, msg: &ServerResponse) -> Result<()> {
    let mut writer = writer.lock().await;
    // 将 ServerResponse 枚举序列化为 JSON 字符串
    let json = serde_json::to_string(msg)?;
    // println!("发送消息: {}", json);
    
    // 将序列化后的数据长度写入流中（以便接收方知道数据大小）
    let length = json.len() as u32;
    writer.write_all(&length.to_be_bytes()).await?;
    
    // 写入序列化后的 JSON 数据
    writer.write_all(json.as_bytes()).await?;

    writer.flush().await?; // 确保刷新缓冲区
    
    Ok(())
}

// 实现 reader_packet 函数
pub async fn reader_packet(reader: &mut Arc<Mutex<tokio::io::BufReader<tokio::net::tcp::OwnedReadHalf>>>) -> Result<ClientRequest> {
    let mut reader = reader.lock().await;
    // 读取数据长度（4 字节大端）
    let mut length_buf = [0u8; 4];
    reader.read_exact(&mut length_buf).await?;
    let length = u32::from_be_bytes(length_buf) as usize;
    
    // 根据数据长度读取 JSON 数据
    let mut json_buf = vec![0u8; length];
    reader.read_exact(&mut json_buf).await?;
    // println!("接收消息: {}", String::from_utf8(json_buf.clone())?);
    
    // 将读取到的 JSON 数据反序列化为 ClientRequest 枚举
    let request = serde_json::from_slice(&json_buf)?;
    
    Ok(request)
}