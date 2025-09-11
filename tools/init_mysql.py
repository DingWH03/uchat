import os
import pymysql

# --- 读取 config.toml ---
try:
    import tomllib  # Python 3.11+
except ModuleNotFoundError:
    import tomli as tomllib  # pip install tomli

CONFIG_PATH = os.getenv("CONFIG_PATH", "config.toml")

def load_config(path: str) -> dict:
    with open(path, "rb") as f:
        cfg = tomllib.load(f)
    return cfg

def parse_mysql_url(url: str) -> dict:
    """
    解析形如 mysql://user:pass@host:port/db 的连接串
    """
    if not url.startswith("mysql://"):
        raise ValueError("DATABASE.url 必须以 mysql:// 开头")

    body = url.replace("mysql://", "", 1)
    if "@" not in body or "/" not in body:
        raise ValueError("DATABASE.url 格式错误，应为 mysql://user:pass@host:port/db")

    credentials, host_db = body.split("@", 1)
    if ":" not in credentials:
        raise ValueError("DATABASE.url 缺少密码部分，应为 user:password")
    user, password = credentials.split(":", 1)

    host_port, db = host_db.split("/", 1)
    if ":" in host_port:
        host, port = host_port.split(":", 1)
        port = int(port)
    else:
        host, port = host_port, 3306

    return {
        "user": user,
        "password": password,
        "host": host,
        "port": port,
        "database": db,
    }

cfg = load_config(CONFIG_PATH)

# --- 数据库配置 ---
db_type = (cfg.get("database", {}).get("type") or "").lower()
db_url = cfg.get("database", {}).get("url")
if db_type != "mysql":
    raise ValueError("仅支持 MySQL：请在 [database] 中设置 type = \"mysql\"")
if not db_url:
    raise ValueError("[database].url 未设置")

db_config = parse_mysql_url(db_url)
print("解析成功：", db_config)

# --- SQL 定义 ---
SQL_CREATE_TABLES = [
    # users
    """
    CREATE TABLE IF NOT EXISTS users (
        id INT UNSIGNED PRIMARY KEY AUTO_INCREMENT,
        username VARCHAR(255) NOT NULL,
        password_hash VARCHAR(255) NOT NULL,
        role ENUM('admin', 'user') NOT NULL DEFAULT 'user',
        bio VARCHAR(256) DEFAULT NULL,
        avatar_url VARCHAR(255) DEFAULT NULL,
        friends_updated_at DATETIME DEFAULT CURRENT_TIMESTAMP,
        groups_updated_at DATETIME DEFAULT CURRENT_TIMESTAMP
    );
    """,
    # messages (private)
    """
    CREATE TABLE IF NOT EXISTS messages (
        id BIGINT UNSIGNED PRIMARY KEY AUTO_INCREMENT,
        sender_id INT UNSIGNED NOT NULL,
        receiver_id INT UNSIGNED NOT NULL,
        message_type ENUM('text', 'image', 'file', 'video', 'audio') NOT NULL,
        message TEXT NOT NULL,
        timestamp BIGINT DEFAULT 0 NOT NULL,
        FOREIGN KEY (sender_id) REFERENCES users(id) ON DELETE CASCADE,
        FOREIGN KEY (receiver_id) REFERENCES users(id) ON DELETE CASCADE,
        INDEX idx_sender_receiver_time (sender_id, receiver_id, timestamp),
        INDEX idx_receiver_time (receiver_id, timestamp)
    );
    """,
    # friendships
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
    # ugroups
    """
    CREATE TABLE IF NOT EXISTS ugroups (
        id INT UNSIGNED PRIMARY KEY AUTO_INCREMENT,
        name VARCHAR(255) NOT NULL,
        creator_id INT UNSIGNED NOT NULL,
        description VARCHAR(256) DEFAULT NULL,
        avatar_url VARCHAR(255) DEFAULT NULL,
        created_at DATETIME DEFAULT CURRENT_TIMESTAMP,
        FOREIGN KEY (creator_id) REFERENCES users(id) ON DELETE CASCADE
    );
    """,
    # group_members
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
    # ugroup_messages
    """
    CREATE TABLE IF NOT EXISTS ugroup_messages (
        id BIGINT UNSIGNED PRIMARY KEY AUTO_INCREMENT,
        group_id INT UNSIGNED NOT NULL,
        sender_id INT UNSIGNED NOT NULL,
        message_type ENUM('text', 'image', 'file', 'video', 'audio') NOT NULL,
        message TEXT NOT NULL,
        timestamp BIGINT DEFAULT 0 NOT NULL,
        FOREIGN KEY (group_id) REFERENCES ugroups(id) ON DELETE CASCADE,
        FOREIGN KEY (sender_id) REFERENCES users(id) ON DELETE CASCADE,
        INDEX idx_group_time (group_id, timestamp),
        INDEX idx_sender_group_time (sender_id, group_id, timestamp)
    );
    """,
    # view（用 OR REPLACE，避免重复创建报错）
    """
    CREATE OR REPLACE VIEW recent_private_messages_view AS
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
    """,
]

# 触发器：用 DROP IF EXISTS + CREATE，兼容性更好
SQL_TRIGGERS = [
    # friendships insert
    "DROP TRIGGER IF EXISTS trg_friendships_insert;",
    """
    CREATE TRIGGER trg_friendships_insert
    AFTER INSERT ON friendships
    FOR EACH ROW
    BEGIN
        UPDATE users SET friends_updated_at = NOW() WHERE id = NEW.user_id;
        UPDATE users SET friends_updated_at = NOW() WHERE id = NEW.friend_id;
    END;
    """,
    # friendships delete
    "DROP TRIGGER IF EXISTS trg_friendships_delete;",
    """
    CREATE TRIGGER trg_friendships_delete
    AFTER DELETE ON friendships
    FOR EACH ROW
    BEGIN
        UPDATE users SET friends_updated_at = NOW() WHERE id = OLD.user_id;
        UPDATE users SET friends_updated_at = NOW() WHERE id = OLD.friend_id;
    END;
    """,
    # group_members insert
    "DROP TRIGGER IF EXISTS trg_group_members_insert;",
    """
    CREATE TRIGGER trg_group_members_insert
    AFTER INSERT ON group_members
    FOR EACH ROW
    BEGIN
        UPDATE users SET groups_updated_at = NOW() WHERE id = NEW.user_id;
    END;
    """,
    # group_members delete
    "DROP TRIGGER IF EXISTS trg_group_members_delete;",
    """
    CREATE TRIGGER trg_group_members_delete
    AFTER DELETE ON group_members
    FOR EACH ROW
    BEGIN
        UPDATE users SET groups_updated_at = NOW() WHERE id = OLD.user_id;
    END;
    """,
]

def connect_db():
    return pymysql.connect(
        host=db_config["host"],
        user=db_config["user"],
        password=db_config["password"],
        database=db_config["database"],
        port=db_config["port"],
        charset="utf8mb4",
        cursorclass=pymysql.cursors.DictCursor,
        autocommit=False,
    )

def create_tables():
    try:
        connection = connect_db()
        print("成功连接到数据库")
        with connection.cursor() as cursor:
            # 基础表/视图
            for q in SQL_CREATE_TABLES:
                cursor.execute(q)
                first_line = q.strip().splitlines()[0]
                print(f"执行成功：{first_line}")

            # 自增起始值
            cursor.execute("ALTER TABLE users AUTO_INCREMENT = 10000000;")
            cursor.execute("ALTER TABLE ugroups AUTO_INCREMENT = 10000000;")

            # 触发器（用单条 execute 即可，PyMySQL 会把整段发送给服务器）
            for q in SQL_TRIGGERS:
                cursor.execute(q)
                first_line = q.strip().splitlines()[0]
                print(f"执行成功：{first_line}")

            connection.commit()
        print("所有表与触发器已创建/更新完成")
    except Exception as e:
        print("发生错误：", e)
        try:
            connection.rollback()
        except Exception:
            pass
    finally:
        try:
            connection.close()
        except Exception:
            pass

if __name__ == "__main__":
    create_tables()
