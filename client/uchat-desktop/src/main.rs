use tokio;
use anyhow::Result;
use uchat_coreapi::core_api::CoreApi; // 替换为正确的模块路径
use uchat_coreapi::protocol::{ServerResponse}; // 替换为协议模块路径

#[tokio::main]
async fn main() -> Result<()> {
    // 1. 建立客户端并连接服务器
    let core_api = CoreApi::new("127.0.0.1", 8080).await?;

    // 2. 启动监听队列的逻辑，不阻塞后续操作
    core_api.listen_messages(|resp| {
        // 在这里可以进行进一步处理（比如更新UI、刷新列表等等）
        println!("(回调) 收到消息: {:?}", resp);
    }).await;

    // 3. 随时发请求，而不受监听干扰
    // core_api.send_register("alice", "password").await?;
    core_api.send_login(3, "password").await?;

    // 比如获取在线用户
    core_api.get_online_users().await?;

    // 4. 模拟运行，不退出程序
    loop {
        tokio::time::sleep(tokio::time::Duration::from_secs(10)).await;
        println!("客户端还在运行...");
    }
}
