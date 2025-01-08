use std::sync::Arc;

use anyhow::Result;
use tokio::runtime::Runtime;

use uchat_coreapi::core_api::CoreApi;
use uchat_coreapi::protocol::ServerResponse;

use cxx_qt::CxxQtThread;
use cxx_qt::QtCore::{QGuiApplication, QQmlApplicationEngine};

#[cxx_qt::bridge]
mod gui_object {
    use super::*;

    /// 这是我们的 Qt 对象，用于和 QML 交互
    #[derive(Default)]
    pub struct MyGuiObject {
        /// 用于与服务器通信的 API
        core_api: Option<Arc<CoreApi>>,
        /// 用于在后台执行异步操作（Tokio 线程池）
        runtime: Option<Arc<Runtime>>,
    }

    /// `qobject::MyGuiObject` 的实现
    #[cxx_qt::qobject]
    impl MyGuiObject {
        /// QML 按钮调用此方法执行登录
        #[qinvokable]
        pub fn do_login(self: Pin<&mut Self>, user_id_text: &str, password: &str) {
            let user_id_result = user_id_text.parse::<u32>();
            if let (Some(core_api), Some(rt)) = (self.core_api.clone(), self.runtime.clone()) {
                match user_id_result {
                    Ok(user_id) => {
                        rt.spawn(async move {
                            if let Err(e) = core_api.send_login(user_id, password).await {
                                eprintln!("发送登录请求失败: {:?}", e);
                            }
                        });
                    }
                    Err(e) => {
                        eprintln!("User ID 无法转换为数字: {:?}", e);
                    }
                }
            } else {
                eprintln!("core_api 或 runtime 尚未初始化");
            }
        }

        /// 请求在线用户列表
        #[qinvokable]
        pub fn get_online_users(self: Pin<&mut Self>) {
            if let (Some(core_api), Some(rt)) = (self.core_api.clone(), self.runtime.clone()) {
                rt.spawn(async move {
                    if let Err(e) = core_api.get_online_users().await {
                        eprintln!("获取在线用户列表失败: {:?}", e);
                    }
                });
            } else {
                eprintln!("core_api 或 runtime 尚未初始化");
            }
        }
    }
}

#[cxx_qt::main]
async fn main() -> Result<()> {
    // 1. 创建 Tokio 多线程 runtime（cxx-qt 已经帮助我们创建，但这里要得到其句柄）
    let rt = tokio::runtime::Handle::current();
    let rt = Arc::new(rt);

    // 2. 创建 CoreApi
    //    注意：如果你的 CoreApi::new(...) 是一个 async 方法，我们需要在此 async 上文执行
    let core_api = Arc::new(CoreApi::new("127.0.0.1", 8080).await?);

    // 3. 启动服务器消息监听
    {
        let core_api_clone = core_api.clone();
        rt.spawn(async move {
            core_api_clone
                .listen_messages(move |resp| {
                    match resp {
                        ServerResponse::ReceiveMessage {
                            sender,
                            message,
                            timestamp,
                        } => {
                            println!("[{}] 用户({})发来消息: {}", timestamp, sender, message);
                        }
                        ServerResponse::Error { message } => {
                            eprintln!("服务端错误: {}", message);
                        }
                        ServerResponse::OnlineUsers { user_ids, .. } => {
                            println!("当前在线用户列表: {:?}", user_ids);
                        }
                        _ => {
                            println!("其他响应: {:?}", resp);
                        }
                    }
                })
                .await;
        });
    }

    // 4. 构建 Qt 应用，并加载 QML 界面
    let mut app = QGuiApplication::new();
    let mut engine = QQmlApplicationEngine::new();

    // 5. 创建 MyGuiObject 并注入 core_api、runtime
    let my_obj = gui_object::MyGuiObject::new();
    {
        let mut my_obj_mut = my_obj.as_mut();
        my_obj_mut.set_core_api(Some(core_api));
        my_obj_mut.set_runtime(Some(rt));
    }

    // 6. 将该对象注入到 QML (context property)，在 QML 中可通过 “myGuiObject” 访问它
    engine.root_context().set_context_property("myGuiObject", my_obj.as_qobject());

    // 7. 加载 QML
    //    - 这里演示把 QML 直接内联进来，也可使用 `engine.load_file("main.qml".into())`
    let qml_data = include_str!("../resources/main.qml");
    engine.load_data(QString::from(qml_data));

    // 8. 启动 Qt 事件循环
    engine.exec();
    drop(engine);
    drop(app);

    Ok(())
}
