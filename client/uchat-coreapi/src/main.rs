use uchat_coreapi::core_api::{ConnectionState, CoreApi};
use uchat_coreapi::protocol::ServerResponse;
use tokio::time::Duration;
use tokio::time::sleep;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // 创建 CoreApi 实例
    let core_api = CoreApi::new("127.0.0.1", 8080).await?;
    println!("CoreApi 初始化成功，连接到服务器");

    // 启动消息监听任务
    let core_api_clone = core_api.clone();
    tokio::spawn(async move {
        core_api_clone.listen_messages(|response| {
            match response {
                ServerResponse::ReceiveMessage { group_id, sender, message, timestamp } => {
                    println!(
                        "[{}] 收到来自用户 {} 的消息 (群组 {}): {}",
                        timestamp, sender, group_id, message
                    );
                }
                ServerResponse::Error { message } => {
                    eprintln!("服务器返回错误: {}", message);
                }
                ServerResponse::OnlineUsers { user_ids, .. } => {
                    println!("当前在线用户列表: {:?}", user_ids);
                }
                _ => {
                    println!("收到其他类型响应: {:?}", response);
                }
            }
        }).await;
    });

    // // 模拟用户注册
    // println!("尝试注册用户...");
    // core_api
    //     .send_register("test_user", "test_password")
    //     .await
    //     .expect("注册失败");
    // println!("用户注册成功");

    // // 等待服务器处理注册请求
    // sleep(Duration::from_secs(1)).await;

    // 模拟用户登录
    println!("尝试登录...");
    core_api
        .send_login(5, "test_password") // 假设注册后分配的 user_id 是 1
        .await
        .expect("登录失败");
    println!("用户登录成功");

    // 获取当前状态
    let state = core_api.get_state().await;
    println!("当前连接状态: {:?}", state);

    // 模拟发送消息
    println!("尝试发送消息...");
    core_api
        .send_message(0, 2, "你好，这是测试消息") // 假设发送给用户 2
        .await
        .expect("发送消息失败");
    println!("消息发送成功");

    // 模拟获取在线用户列表
    println!("尝试获取在线用户列表...");
    core_api
        .get_online_users()
        .await
        .expect("获取在线用户列表失败");
    println!("已请求在线用户列表");

    // 模拟持续运行，监听服务器消息
    loop {
        sleep(Duration::from_secs(5)).await;

        // 获取当前状态
        let state = core_api.get_state().await;
        println!("当前连接状态: {:?}", state);

        // 如果连接中断，尝试重新连接
        if state == ConnectionState::LoggedInButDisconnected {
            println!("检测到连接中断，尝试重新连接...");
            let core_api = CoreApi::new("127.0.0.1", 8080).await?;
            println!("重新连接成功");
        }
    }
}