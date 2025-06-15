use axum::{
    extract::{Extension, Query},
    response::{Html, IntoResponse},
};
use crate::{server::AppState};

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

    let mut html = String::from(r#"
        <html>
        <head>
            <title>UChat 管理面板</title>
            <style>
                body { font-family: sans-serif; padding: 20px; }
                .user { margin-top: 10px; font-weight: bold; }
                .session { margin-left: 20px; color: #666; }
                button { margin: 5px; }
            </style>
        </head>
        <body>
            <h1>在线用户列表</h1>
            <form method="get">
                <button name="action" value="clear_all">清除所有会话</button>
            </form>
    "#);

    for (user_id, sessions) in tree {
        html += &format!(r#"<div class="user">用户 ID: {} <form method="get" style="display:inline"><input type="hidden" name="user_id" value="{}"><button name="action" value="remove_user">移除该用户会话</button></form></div>"#, user_id, user_id);
        for (session_id, info) in sessions {
            html += &format!(
                r#"<div class="session">Session ID: {} | IP: {:?} | 创建时间: {}</div>"#,
                session_id,
                info.ip,
                info.created_at
            );
        }
    }

    html += "</body></html>";

    Html(html)
}
