-- レコードテーブル
CREATE TABLE IF NOT EXISTS records (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    domain_pattern TEXT NOT NULL,
    record_type TEXT NOT NULL,
    content TEXT NOT NULL,
    ttl INTEGER NOT NULL DEFAULT 60,
    active INTEGER NOT NULL DEFAULT 1
);

-- クエリログテーブル
CREATE TABLE IF NOT EXISTS query_logs (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    query_name TEXT NOT NULL,
    q_type TEXT NOT NULL,
    result_type TEXT NOT NULL,
    duration_ms INTEGER NOT NULL,
    timestamp DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP
);

-- 設定テーブル
CREATE TABLE IF NOT EXISTS settings (
    key TEXT PRIMARY KEY,
    value TEXT NOT NULL
);

-- 初期設定値の投入
INSERT OR IGNORE INTO settings (key, value) VALUES ('upstream_primary', '8.8.8.8:53');
INSERT OR IGNORE INTO settings (key, value) VALUES ('upstream_secondary', '1.1.1.1:53');
INSERT OR IGNORE INTO settings (key, value) VALUES ('upstream_timeout_ms', '2000');
INSERT OR IGNORE INTO settings (key, value) VALUES ('log_retention_days', '7');

-- インデックス作成
CREATE INDEX IF NOT EXISTS idx_records_active ON records(active);
CREATE INDEX IF NOT EXISTS idx_query_logs_timestamp ON query_logs(timestamp);
