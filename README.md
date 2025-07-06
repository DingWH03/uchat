# Uchat

这是一个完全由 Rust 语言打造的实时通讯后端平台。

## 特性

- 基于 axum 框架
- 使用session_id提供身份验证
- 模块化，便于扩展

## 快速开始

### 依赖环境

- Rust 版本 >= 1.75
- Mysql 或 PostgreSql
- Minio
- Redis

### 配置文件

需要准备好数据库、minio 对象存储和 Redis 高性能数据库，将 URL 全部写在配置文件 `.env` 中。

```bash
cp .env.example .env
```

然后对其中的配置项进行修改即可。

完成环境变量的修改后，需要预先对数据库进行建库操作，并使用 `tools/init_mysql.py` 进行基础表的创建。

```bash
python -m pip install pymysql
python -m pip install dotenv
python tools/init_mysql.py
```

### 构建 & 运行

```bash
# 克隆项目
git clone https://github.com/DingWH03/uchat.git
cd uchat

# 编译运行
RUST_LOG=debug cargo run --bin=uchat-server
```

### API 文档

server 运行后，访问 `SERVER_URL/swagger` 可以访问 swagger-ui 在线查看文档，也可以查看预生成的[openapi.json](doc/api/openapi.json)，openapi.json 转换的md文件[openapi.md](doc/api/openapi.md)。

## 目录结构

```text
.
├── bin
│   ├── client              # 未使用的客户端程序
│   │   ├── Cargo.toml
│   │   └── src
│   │       ├── client.rs
│   │       └── main.rs
│   └── server              # 服务端核心程序
│       ├── build.rs
│       ├── Cargo.toml
│       └── src
│           ├── api         # manager和request实际处理过程和handler函数
│           ├── db          # 数据库抽象层
│           ├── error.rs    # 错误处理
│           ├── main.rs     # 主程序
│           ├── redis       # Redis抽象层
│           ├── server      # axum server
│           ├── session     # 会话管理
│           └── storage     # 对象存储抽象层
├── Cargo.lock
├── Cargo.toml
├── CHANGELOG.md
├── CONTRIBUTING.md
├── crate
│   └── uchat_protocol      # 数据模型以及api接口定义模块
│       ├── Cargo.toml
│       └── src
│           ├── frame.rs
│           ├── lib.rs
│           ├── manager
│           ├── message.rs
│           ├── model.rs
│           ├── mod.rs
│           └── request
├── doc
│   └── api                 # api文档
│       ├── openapi.json
│       └── openapi.md
├── LICENSE
├── README.md
└── tools                   # python工具
    ├── apidoc2md.py
    ├── init_mysql.py
    ├── init_postgresql.py
    └── README.md
```

## 贡献

欢迎提 PR、Issue！
详见 [CONTRIBUTING.md](./CONTRIBUTING.md)

## License

本项目采用 GPL-V3 License - 详见 [LICENSE](./LICENSE)
