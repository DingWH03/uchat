use crate::server::AppState;
use axum::{
    extract::{Extension, Query},
    response::{Html, IntoResponse},
};

#[derive(Debug, serde::Deserialize)]
pub struct AdminAction {
    action: Option<String>,
    user_id: Option<u32>,
}

pub async fn handle_admin_dashboard(
    Extension(state): Extension<AppState>,
    Query(query): Query<AdminAction>,
) -> impl IntoResponse {
    let manager = state.manager.lock().await;

    // 处理控制操作
    if let Some(action) = query.action.as_deref() {
        match action {
            "clear_all" => {
                manager.remove_all_sessions().await;
            }
            "remove_user" => {
                if let Some(user_id) = query.user_id {
                    manager.remove_user_sessions(user_id).await;
                }
            }
            _ => {}
        }
    }

    // 获取在线用户树
    let tree = manager.get_online_user_tree().await;

    let mut html = String::from(
        r#"
        <!DOCTYPE html>
        <html lang="zh">
        <head>
            <meta charset="UTF-8">
            <title>UChat 管理面板</title>
            <style>
                * {
                    box-sizing: border-box;
                }
                body {
                    font-family: 'Segoe UI', Tahoma, Geneva, Verdana, sans-serif;
                    background-color: #f4f6f8;
                    color: #333;
                    padding: 30px;
                    margin: 0;
                }
                h1 {
                    text-align: center;
                    color: #2c3e50;
                }
                .container {
                    max-width: 900px;
                    margin: 0 auto;
                }
                .control-panel {
                    text-align: center;
                    margin-bottom: 30px;
                }
                .control-panel button {
                    background-color: #e74c3c;
                    color: white;
                    border: none;
                    padding: 10px 20px;
                    border-radius: 6px;
                    cursor: pointer;
                    font-size: 16px;
                    margin: 5px;
                    transition: background-color 0.2s ease;
                }
                .control-panel button:hover {
                    background-color: #c0392b;
                }
                .user-card {
                    background-color: #ffffff;
                    border-radius: 8px;
                    box-shadow: 0 2px 8px rgba(0, 0, 0, 0.1);
                    padding: 20px;
                    margin-bottom: 20px;
                }
                .user-header {
                    font-size: 18px;
                    font-weight: bold;
                    margin-bottom: 10px;
                    display: flex;
                    justify-content: space-between;
                    align-items: center;
                }
                .user-header form {
                    margin: 0;
                }
                .user-header button {
                    background-color: #3498db;
                    color: white;
                    border: none;
                    padding: 6px 12px;
                    border-radius: 4px;
                    cursor: pointer;
                    font-size: 14px;
                }
                .user-header button:hover {
                    background-color: #2980b9;
                }
                .session {
                    margin-left: 10px;
                    font-size: 14px;
                    color: #555;
                    line-height: 1.6;
                }
            </style>
        </head>
        <body>
            <div class="container">
                <h1>UChat 在线用户列表</h1>
                <div class="control-panel">
                    <form method="get">
                        <button name="action" value="clear_all">清除所有会话</button>
                    </form>
                </div>
    "#,
    );

    for (user_id, sessions) in tree {
        html += &format!(
            r#"<div class="user-card">
                <div class="user-header">
                    用户 ID: {}
                    <form method="get">
                        <input type="hidden" name="user_id" value="{}">
                        <button name="action" value="remove_user">移除该用户会话</button>
                    </form>
                </div>"#,
            user_id, user_id
        );
        for (session_id, info) in sessions {
            html += &format!(
                r#"<div class="session">Session ID: {} | IP: {:?} | 创建时间: {}</div>"#,
                session_id,
                info.ip,
                info.created_at.format("%Y-%m-%d %H:%M:%S") // 若为 chrono::DateTime
            );
        }
        html += "</div>";
    }

    html += "</div></body></html>";

    Html(html)
}
