import pymysql
from dotenv import load_dotenv
import os

# 加载环境变量
load_dotenv(".env")

# 从 .env 文件读取 DATABASE_URL
DATABASE_URL = os.getenv("DATABASE_URL")

if not DATABASE_URL:
    raise ValueError("DATABASE_URL 环境变量未设置")

# 解析 DATABASE_URL
db_config = {}
if DATABASE_URL.startswith("mysql://"):
    parts = DATABASE_URL.replace("mysql://", "").split("@")
    credentials, host_db = parts[0], parts[1]
    user, password = credentials.split(":")
    host_port, db = host_db.split("/")
    if ":" in host_port:
        host, port = host_port.split(":")
    else:
        host, port = host_port, 3306  # 默认端口 3306
    db_config = {
        "user": user,
        "password": password,
        "host": host,
        "port": int(port),
        "database": db,
    }
    print("解析成功：", db_config)
else:
    raise ValueError("DATABASE_URL 格式不正确")

# 数据库表创建 SQL
SQL_QUERIES = [
    # 用户表
    """
    CREATE TABLE IF NOT EXISTS users (
        id INT UNSIGNED PRIMARY KEY AUTO_INCREMENT,
        username VARCHAR(255) NOT NULL,
        password_hash VARCHAR(255) NOT NULL,
        role ENUM('admin', 'user') NOT NULL DEFAULT 'user',
        bio VARCHAR(256) DEFAULT NULL, -- 个人简介，最多 256 字符
        avatar_url VARCHAR(255) DEFAULT NULL, -- 头像 URL，存储头像在 MinIO 的链接
        friends_updated_at DATETIME DEFAULT CURRENT_TIMESTAMP,
        groups_updated_at DATETIME DEFAULT CURRENT_TIMESTAMP
    );
    """,

    # 消息表（私聊）
    """
    CREATE TABLE IF NOT EXISTS messages (
        id INT UNSIGNED PRIMARY KEY AUTO_INCREMENT,
        sender_id INT UNSIGNED NOT NULL,
        receiver_id INT UNSIGNED NOT NULL,
        message_type ENUM('text', 'image', 'file', 'video', 'audio') NOT NULL,
        message TEXT NOT NULL,
        timestamp BIGINT DEFAULT 0 NOT NULL, -- 使用 BIGINT 存储时间戳，单位为秒
        FOREIGN KEY (sender_id) REFERENCES users(id) ON DELETE CASCADE,
        FOREIGN KEY (receiver_id) REFERENCES users(id) ON DELETE CASCADE,

        -- 索引优化：加快用户聊天记录分页、时间查询
        INDEX idx_sender_receiver_time (sender_id, receiver_id, timestamp),
        INDEX idx_receiver_time (receiver_id, timestamp)
    );
    """,

    # 好友关系表
    """
    CREATE TABLE IF NOT EXISTS friendships (
        user_id INT UNSIGNED NOT NULL,
        friend_id INT UNSIGNED NOT NULL,
        created_at DATETIME DEFAULT CURRENT_TIMESTAMP,
        PRIMARY KEY (user_id, friend_id),
        FOREIGN KEY (user_id) REFERENCES users(id) ON DELETE CASCADE,
        FOREIGN KEY (friend_id) REFERENCES users(id) ON DELETE CASCADE
    );
    """,

    # 群聊表
    """
    CREATE TABLE IF NOT EXISTS ugroups (
        id INT UNSIGNED PRIMARY KEY AUTO_INCREMENT,
        name VARCHAR(255) NOT NULL,
        creator_id INT UNSIGNED NOT NULL,  -- 创建者id
        description VARCHAR(256) DEFAULT NULL, -- 群聊简介，最多 256 字符
        avatar_url VARCHAR(255) DEFAULT NULL, -- 群聊头像 URL
        created_at DATETIME DEFAULT CURRENT_TIMESTAMP,
        FOREIGN KEY (creator_id) REFERENCES users(id) ON DELETE CASCADE
    );
    """,

    # 群聊成员表
    """
    CREATE TABLE IF NOT EXISTS group_members (
        group_id INT UNSIGNED NOT NULL,
        user_id INT UNSIGNED NOT NULL,
        joined_at DATETIME DEFAULT CURRENT_TIMESTAMP,
        PRIMARY KEY (group_id, user_id),
        FOREIGN KEY (group_id) REFERENCES ugroups(id) ON DELETE CASCADE,
        FOREIGN KEY (user_id) REFERENCES users(id) ON DELETE CASCADE,

        INDEX idx_user_id (user_id)
    );
    """,

    # 群聊消息表
    """
    CREATE TABLE IF NOT EXISTS ugroup_messages (
        id INT UNSIGNED PRIMARY KEY AUTO_INCREMENT,
        group_id INT UNSIGNED NOT NULL,
        sender_id INT UNSIGNED NOT NULL,
        message_type ENUM('text', 'image', 'file', 'video', 'audio') NOT NULL,
        message TEXT NOT NULL,
        timestamp BIGINT DEFAULT 0 NOT NULL,
        FOREIGN KEY (group_id) REFERENCES ugroups(id) ON DELETE CASCADE,
        FOREIGN KEY (sender_id) REFERENCES users(id) ON DELETE CASCADE,

        -- 索引优化：分页加载、获取某群最新消息
        INDEX idx_group_time (group_id, timestamp),
        INDEX idx_sender_group_time (sender_id, group_id, timestamp)
    );
    """,

    # 聊天记录视图
    """
    CREATE VIEW recent_private_messages_view AS
    SELECT 
        m.id,
        m.sender_id,
        sender.username AS sender_username,
        m.receiver_id,
        receiver.username AS receiver_username,
        m.message_type,
        LEFT(m.message, 100) AS message_preview,
        m.timestamp
    FROM 
        messages AS m
    JOIN users AS sender ON m.sender_id = sender.id
    JOIN users AS receiver ON m.receiver_id = receiver.id;
    """
]

# 在 SQL_QUERIES 的末尾追加
SQL_QUERIES.extend([
    """
    CREATE TRIGGER IF NOT EXISTS trg_friendships_insert AFTER INSERT ON friendships FOR EACH ROW
    BEGIN
        UPDATE users SET friends_updated_at = NOW() WHERE id = NEW.user_id;
        UPDATE users SET friends_updated_at = NOW() WHERE id = NEW.friend_id;
    END;
    """,

    """
    CREATE TRIGGER IF NOT EXISTS trg_friendships_delete AFTER DELETE ON friendships FOR EACH ROW
    BEGIN
        UPDATE users SET friends_updated_at = NOW() WHERE id = OLD.user_id;
        UPDATE users SET friends_updated_at = NOW() WHERE id = OLD.friend_id;
    END;
    """,

    """
    CREATE TRIGGER IF NOT EXISTS trg_group_members_insert AFTER INSERT ON group_members FOR EACH ROW
    BEGIN
        UPDATE users SET groups_updated_at = NOW() WHERE id = NEW.user_id;
    END;
    """,

    """
    CREATE TRIGGER IF NOT EXISTS trg_group_members_delete AFTER DELETE ON group_members FOR EACH ROW
    BEGIN
        UPDATE users SET groups_updated_at = NOW() WHERE id = OLD.user_id;
    END;
    """
])



def connect_db():
    """
    连接数据库并返回连接对象
    """
    return pymysql.connect(
        host=db_config["host"],
        user=db_config["user"],
        password=db_config["password"],
        database=db_config["database"],
        charset="utf8mb4",
        cursorclass=pymysql.cursors.DictCursor
    )


def create_tables():
    """
    执行 SQL_QUERIES 中的建表语句
    """
    try:
        connection = connect_db()
        print("成功连接到数据库")

        with connection.cursor() as cursor:
            for query in SQL_QUERIES:
                cursor.execute(query)
                print(f"执行成功：{query.splitlines()[1].strip()}")  # 打印 SQL 的第一行
            # 设置 users 和 ugroups 的自增起始值
            cursor.execute("ALTER TABLE users AUTO_INCREMENT = 10000000;")
            cursor.execute("ALTER TABLE ugroups AUTO_INCREMENT = 10000000;")

            connection.commit()
        print("所有表已成功创建")
    except Exception as e:
        print("发生错误：", e)
    finally:
        connection.close()


if __name__ == "__main__":
    create_tables()
