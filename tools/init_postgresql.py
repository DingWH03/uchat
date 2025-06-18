import psycopg
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
if DATABASE_URL.startswith("postgresql://"):
    parts = DATABASE_URL.replace("postgresql://", "").split("@")
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
        id SERIAL PRIMARY KEY,
        username VARCHAR(255) NOT NULL,
        password_hash VARCHAR(255) NOT NULL,
        role TEXT NOT NULL CHECK (role IN ('user', 'admin')) DEFAULT 'user',
        bio VARCHAR(256),
        avatar_url VARCHAR(255)
    );
    """,

    # 消息表（私聊）
    """
    CREATE TABLE IF NOT EXISTS messages (
        id SERIAL PRIMARY KEY,
        sender_id INTEGER NOT NULL,
        receiver_id INTEGER NOT NULL,
        message_type TEXT NOT NULL CHECK (message_type IN ('text', 'image', 'file', 'video', 'audio')),
        message TEXT NOT NULL,
        timestamp TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
        FOREIGN KEY (sender_id) REFERENCES users(id) ON DELETE CASCADE,
        FOREIGN KEY (receiver_id) REFERENCES users(id) ON DELETE CASCADE
    );
    """,

    # 好友关系表
    """
    CREATE TABLE IF NOT EXISTS friendships (
        user_id INTEGER NOT NULL,
        friend_id INTEGER NOT NULL,
        created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
        PRIMARY KEY (user_id, friend_id),
        FOREIGN KEY (user_id) REFERENCES users(id) ON DELETE CASCADE,
        FOREIGN KEY (friend_id) REFERENCES users(id) ON DELETE CASCADE
    );
    """,

    # 群聊表
    """
    CREATE TABLE IF NOT EXISTS ugroups (
        id SERIAL PRIMARY KEY,
        name VARCHAR(255) NOT NULL,
        creator_id INTEGER NOT NULL,
        description VARCHAR(256),
        avatar_url VARCHAR(255),
        created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
        FOREIGN KEY (creator_id) REFERENCES users(id) ON DELETE CASCADE
    );
    """,

    # 群聊成员表
    """
    CREATE TABLE IF NOT EXISTS group_members (
        group_id INTEGER NOT NULL,
        user_id INTEGER NOT NULL,
        joined_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
        PRIMARY KEY (group_id, user_id),
        FOREIGN KEY (group_id) REFERENCES ugroups(id) ON DELETE CASCADE,
        FOREIGN KEY (user_id) REFERENCES users(id) ON DELETE CASCADE
    );
    """,

    # 群聊消息表
    """
    CREATE TABLE IF NOT EXISTS ugroup_messages (
        id SERIAL PRIMARY KEY,
        group_id INTEGER NOT NULL,
        sender_id INTEGER NOT NULL,
        message TEXT NOT NULL,
        timestamp TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
        FOREIGN KEY (group_id) REFERENCES ugroups(id) ON DELETE CASCADE,
        FOREIGN KEY (sender_id) REFERENCES users(id) ON DELETE CASCADE
    );
    """,

    # 离线消息表
    """
    CREATE TABLE IF NOT EXISTS offline_messages (
        id SERIAL PRIMARY KEY,
        receiver_id INTEGER NOT NULL,
        is_group BOOLEAN NOT NULL DEFAULT FALSE,
        message_id INTEGER,
        group_message_id INTEGER,
        delivered BOOLEAN DEFAULT FALSE,
        timestamp TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
        FOREIGN KEY (receiver_id) REFERENCES users(id) ON DELETE CASCADE,
        FOREIGN KEY (message_id) REFERENCES messages(id) ON DELETE CASCADE,
        FOREIGN KEY (group_message_id) REFERENCES ugroup_messages(id) ON DELETE CASCADE,
        CHECK (
            (is_group = FALSE AND message_id IS NOT NULL AND group_message_id IS NULL) OR
            (is_group = TRUE AND group_message_id IS NOT NULL AND message_id IS NULL)
        )
    );
    """
]

INDEX_QUERIES = [
    "CREATE INDEX IF NOT EXISTS idx_sender_receiver_time ON messages (sender_id, receiver_id, timestamp);",
    "CREATE INDEX IF NOT EXISTS idx_receiver_time ON messages (receiver_id, timestamp);",
    "CREATE INDEX IF NOT EXISTS idx_user_id ON group_members (user_id);",
    "CREATE INDEX IF NOT EXISTS idx_group_time ON ugroup_messages (group_id, timestamp);",
    "CREATE INDEX IF NOT EXISTS idx_sender_group_time ON ugroup_messages (sender_id, group_id, timestamp);",
    "CREATE INDEX IF NOT EXISTS idx_receiver_undelivered ON offline_messages (receiver_id, delivered, is_group, timestamp);"
]




def connect_db():
    """
    连接 PostgreSQL 数据库并返回连接对象
    """
    return psycopg.connect(
        host=db_config["host"],
        user=db_config["user"],
        password=db_config["password"],
        dbname=db_config["database"],
        port=db_config["port"],
        row_factory=psycopg.rows.dict_row  # 返回 dict 类型结果
    )



def create_tables():
    """
    执行 SQL_QUERIES 中的建表语句与索引创建语句
    """
    try:
        connection = connect_db()
        print("成功连接到数据库")

        with connection.cursor() as cursor:
            for query in SQL_QUERIES:
                cursor.execute(query)
                print(f"执行成功：{query.strip().splitlines()[0]}")

            for idx_query in INDEX_QUERIES:
                cursor.execute(idx_query)
                print(f"索引创建成功：{idx_query.strip().split()[2]}")

            # 设置自增起始值
            cursor.execute("ALTER SEQUENCE IF EXISTS users_id_seq RESTART WITH 10000000;")
            cursor.execute("ALTER SEQUENCE IF EXISTS ugroups_id_seq RESTART WITH 10000000;")

            connection.commit()
        print("所有表与索引已成功创建")
    except Exception as e:
        print("发生错误：", e)
    finally:
        connection.close()



if __name__ == "__main__":
    create_tables()
