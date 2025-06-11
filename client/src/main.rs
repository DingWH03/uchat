use futures_util::{SinkExt, StreamExt};
use reqwest::Client;
use std::net::ToSocketAddrs;
use tokio::net::TcpStream;
use tokio_tungstenite::tungstenite::handshake::client::{generate_key, Request};
use tokio_tungstenite::tungstenite::{client::IntoClientRequest, Message};
use tokio_tungstenite::{client_async, WebSocketStream};
use url::Url;

use crate::client::ClientMessage;

mod client;


fn main() {
    let rt = tokio::runtime::Runtime::new().unwrap();
    rt.block_on(async_main());
}

async fn async_main() {
    let login_url = "http://127.0.0.1:8080/auth/login";
    let ws_url = "ws://127.0.0.1:8080/auth/ws";

    // 登录用户
    let client = Client::builder()
        .cookie_store(true)
        .build()
        .unwrap();

    let login_body = serde_json::json!({
        "userid": 5,
        "password": "123456"
    });

    let login_resp = client
        .post(login_url)
        .json(&login_body)
        .send()
        .await
        .unwrap();

    let text = login_resp.text().await.unwrap();
    println!("登录响应: {}", text);

    let session_id = text
        .split("\"message\":\"")
        .nth(1)
        .unwrap()
        .replace("\"", "")
        .replace("}", "")
        .trim()
        .to_string();

    println!("提取到的 session_id: {}", session_id);

    // 使用标准 HTTP headers 建立请求
    let url = Url::parse(ws_url).unwrap();
    let host = url.host_str().unwrap();
    let port = url.port_or_known_default().unwrap();
    let addr = format!("{}:{}", host, port)
        .to_socket_addrs()
        .unwrap()
        .next()
        .unwrap();

    let stream = TcpStream::connect(&addr).await.unwrap();

    // 构建 WebSocket 请求，携带 Cookie
    let mut req = tokio_tungstenite::tungstenite::client::IntoClientRequest::into_client_request(url.as_str()).unwrap();
    req.headers_mut().insert(
        "Cookie",
        format!("session_id={}", session_id)
            .parse()
            .expect("Invalid header"),
    );

    // 使用 client_async 进行 WebSocket 升级
    let (ws_stream, _) = client_async(req, stream)
        .await
        .expect("WebSocket 连接失败");

    println!("WebSocket 连接成功");

    handle_socket(ws_stream).await;
}

async fn handle_socket(mut ws_stream: WebSocketStream<TcpStream>) {
    let (mut write, mut read) = ws_stream.split();

    // 发送测试消息
    write
        .send(Message::Text("你好，服务器！".into()))
        .await
        .unwrap();


    // 构造 ClientMessage::SendMessage 消息
    let msg = ClientMessage::SendMessage {
        receiver: 5,
        message: "Hello".to_string(),
    };
    let msg_text = serde_json::to_string(&msg).unwrap();

    // 发送这条消息
    write.send(Message::Text(msg_text.into())).await.unwrap();

    // 启动一个任务持续发送心跳包
    tokio::spawn(async move {
        loop {
            if let Err(e) = write
                .send(Message::Ping("ping".into()))
                .await
            {
                eprintln!("心跳发送失败: {}", e);
                break;
            }
            tokio::time::sleep(tokio::time::Duration::from_secs(30)).await;
        }
    });

    // 接收响应
    while let Some(msg) = read.next().await {
        match msg {
            Ok(Message::Text(text)) => println!("收到服务器消息: {}", text),
            Ok(_) => {}
            Err(e) => {
                eprintln!("WebSocket 错误: {}", e);
                break;
            }
        }
    }
}
