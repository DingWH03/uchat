use axum::extract::ws::{Message, WebSocket};
use futures::{sink::SinkExt, stream::StreamExt};
use log::{error, debug, info, warn};
use tokio::sync::mpsc; // tokio::sync::Mutex for Request

use crate::{protocol::ClientMessage, server::AppState};

/// 处理 WebSocket 连接的实际逻辑
pub async fn handle_socket(socket: WebSocket, session_id: String, state: AppState) {
    info!("WebSocket 连接已建立，会话ID: {}", &session_id);

    // 分割 WebSocket 连接为发送端和接收端
    let (mut ws_sender, mut ws_receiver) = socket.split();

    // 创建一个 MPSC 通道，用于从其他任务向此 WebSocket 连接发送消息
    let (tx, mut rx) = mpsc::unbounded_channel::<Message>();

    // 克隆 session_id 以便在 tokio::spawn 任务中使用
    let session_id_for_task = session_id.clone();

    // 注册发送端到 SessionManager
    let req_lock = state.request.lock().await; // 获取 Request 的 Mutex 锁 (tokio::sync::Mutex)
    req_lock.register_session(&session_id, tx).await;
    info!("会话 {} 已注册到 SessionManager", session_id);
    drop(req_lock); // 及时释放 Mutex 锁

    // 通知客户端连接成功
    // if let Err(e) = ws_sender
    //     .send(Message::Text(axum::extract::ws::Utf8Bytes::from(format!(
    //         "连接成功，会话ID: {}",
    //         session_id
    //     ))))
    //     .await
    // {
    //     warn!("无法发送连接成功消息到会话ID {}: {}", session_id, e);
    //     // 连接可能已经断开，直接返回
    //     return;
    // }

    debug!("开始监听会话 {} 的后续消息...", session_id);

    // 启动一个独立的 Tokio 任务，用于从 MPSC 接收器接收消息，并将其发送到 WebSocket
    // let ws_sender_clone = ws_sender.clone();
    tokio::spawn(async move {
        while let Some(msg) = rx.recv().await {
            if ws_sender.send(msg).await.is_err() {
                // 如果发送失败，说明 WebSocket 连接已断开
                break;
            }
        }
        info!(
            "MPSC 到 WebSocket 转发任务结束，会话ID: {}",
            &session_id_for_task
        );
    });

    // 循环处理后续的 WebSocket 消息
    while let Some(msg_result) = ws_receiver.next().await {
        match msg_result {
            Ok(Message::Text(text)) => {
                debug!("会话 {} 收到文本消息: {}", session_id, text);
                socket_handle_text(&session_id, &text, state.clone()).await;
            }
            Ok(Message::Binary(bin)) => {
                debug!("会话 {} 收到二进制消息 ({} 字节)", session_id, bin.len());
                // 处理二进制消息
            }
            Ok(Message::Ping(pong)) => {
                debug!("会话 {} 收到 Ping 消息", session_id);
                let request_lock = state.request.lock().await; // 获取 Request 的 Mutex 锁
                request_lock
                    .send_to_session(&session_id, Message::Pong(pong))
                    .await;
            }
            Ok(Message::Pong(_)) => {
                debug!("会话 {} 收到 Pong 消息", session_id);
            }
            Ok(Message::Close(close_frame)) => {
                debug!("会话 {} 收到 Close 消息: {:?}", session_id, close_frame);
                break; // 客户端请求关闭连接
            }
            Err(e) => {
                warn!("会话 {} WebSocket 接收错误: {}", session_id, e);
                break; // 接收错误，退出循环
            }
        }
    }

    // 连接断开后清理 SessionManager
    info!("WebSocket 连接断开。正在清理会话 {}", session_id);
    let req_lock = state.request.lock().await; // 获取 Request 的 Mutex 锁
    req_lock.unregister_session(&session_id).await;
    drop(req_lock);
    info!("WebSocket 处理任务结束，会话ID: {}", session_id);
}

/// 具体详细的处理文本消息逻辑
/// 完成各类信息的处理与转发，并记录数据库
async fn socket_handle_text(session_id: &str, text: &str, state: AppState) {
    match serde_json::from_str::<ClientMessage>(&text) {
        Ok(ClientMessage::SendMessage { receiver, message }) => {
            debug!("私聊：发送给 {}, 内容: {}", receiver, message);

            let req_lock = state.request.lock().await;
            req_lock
                .send_to_user_v2(session_id, receiver, &message)
                .await;
        }

        Ok(ClientMessage::SendGroupMessage { group_id, message }) => {
            debug!("群聊：群号 {}, 内容: {}", group_id, message);
            let req_lock = state.request.lock().await;
            req_lock
                .send_to_group_v2(session_id, group_id, &message)
                .await;
        }
        Err(e) => {
            error!("会话 {} 发送消息 {} 解析消息失败: {}", session_id, text, e);
        }
    }
}
