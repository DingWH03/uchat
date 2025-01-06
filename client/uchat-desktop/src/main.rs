use tokio;
use anyhow::Result;
use uchat_coreapi::core_api::CoreApi; // 替换为正确的模块路径
use uchat_coreapi::protocol::{ServerResponse}; // 替换为协议模块路径

#[tokio::main]
async fn main() -> Result<()> {
    // 初始化 API
    let core_api = CoreApi::new("127.0.0.1", 8080).await?;
    // println!("连接服务器成功！");

    // 登录
    let login_response = core_api.login(1, "123456").await?;
    println!("Login response: {:?}", login_response);

    // 获取当前用户的用户名
    let username_response = core_api.get_my_username().await?;
    println!("My username response: {:?}", username_response);

    // 发送消息
    let send_message_response = core_api.send_message(2, "Hello, this is a test message!").await?;
    println!("Send message response: {:?}", send_message_response);

    // 获取在线用户列表
    let online_users_response = core_api.get_online_users().await?;
    println!("Online users response: {:?}", online_users_response);

    // 检查用户信息
    let check_user_info_response = core_api.check_user_info(1).await?;
    println!("Check user info response: {:?}", check_user_info_response);

    // // 启动监听接收服务器消息
    // core_api.listen(|response| {
    //     println!("Received message: {:?}", response);
    // }).await?;

    // // 等待一些时间以模拟接收消息
    // tokio::time::sleep(tokio::time::Duration::from_secs(10)).await;


    Ok(())
}
