pub mod models;

use anyhow::{Context, Result};
use sqlx::{
    sqlite::{SqliteConnectOptions, SqlitePoolOptions},
    Pool, Sqlite,
};
use std::str::FromStr;
use std::time::Duration;
use tracing::{info, warn};

pub use models::*;

/// データベース接続プール
pub type DbPool = Pool<Sqlite>;

/// データベース接続を初期化
pub async fn init_db(database_url: &str) -> Result<DbPool> {
    info!("データベース接続を初期化中: {}", database_url);

    // リトライロジック（3回、各1秒間隔）
    let mut attempts = 0;
    let max_attempts = 3;

    loop {
        attempts += 1;

        match try_connect(database_url).await {
            Ok(pool) => {
                info!("データベース接続成功");
                return Ok(pool);
            }
            Err(e) if attempts < max_attempts => {
                warn!(
                    "データベース接続失敗 (試行 {}/{}): {}",
                    attempts, max_attempts, e
                );
                tokio::time::sleep(Duration::from_secs(1)).await;
            }
            Err(e) => {
                return Err(e).context(format!(
                    "{}回の試行後もデータベース接続に失敗しました",
                    max_attempts
                ));
            }
        }
    }
}

/// データベース接続を試行
async fn try_connect(database_url: &str) -> Result<DbPool> {
    // SQLite接続オプション設定（ファイルが存在しない場合は作成）
    let connect_options = SqliteConnectOptions::from_str(database_url)
        .context("データベースURL解析に失敗")?
        .create_if_missing(true);

    // 接続プール作成
    let pool = SqlitePoolOptions::new()
        .max_connections(5)
        .connect_with(connect_options)
        .await
        .context("データベース接続プール作成に失敗")?;

    // WALモード有効化
    sqlx::query("PRAGMA journal_mode = WAL")
        .execute(&pool)
        .await
        .context("WALモード有効化に失敗")?;

    // マイグレーション実行
    run_migrations(&pool)
        .await
        .context("マイグレーション実行に失敗")?;

    Ok(pool)
}

/// マイグレーション実行
async fn run_migrations(pool: &DbPool) -> Result<()> {
    info!("マイグレーションを実行中");

    let migration_sql = include_str!("migration.sql");

    // コメントを除去してからセミコロンで分割
    let cleaned_sql: String = migration_sql
        .lines()
        .filter(|line| {
            let trimmed = line.trim();
            !trimmed.is_empty() && !trimmed.starts_with("--")
        })
        .collect::<Vec<&str>>()
        .join("\n");

    // セミコロンで分割して各文を実行
    for statement in cleaned_sql.split(';') {
        let statement = statement.trim();
        if statement.is_empty() {
            continue;
        }

        sqlx::query(statement)
            .execute(pool)
            .await
            .context(format!("SQL実行に失敗: {}", statement))?;
    }

    info!("マイグレーション完了");
    Ok(())
}

/// アクティブなレコードを全て取得
pub async fn get_active_records(pool: &DbPool) -> Result<Vec<Record>> {
    let records = sqlx::query_as::<_, Record>("SELECT * FROM records WHERE active = 1")
        .fetch_all(pool)
        .await
        .context("アクティブレコードの取得に失敗")?;

    Ok(records)
}

/// 全レコードを取得
pub async fn get_all_records(pool: &DbPool) -> Result<Vec<Record>> {
    let records = sqlx::query_as::<_, Record>("SELECT * FROM records ORDER BY id DESC")
        .fetch_all(pool)
        .await
        .context("レコード取得に失敗")?;

    Ok(records)
}

/// レコードをIDで取得
pub async fn get_record_by_id(pool: &DbPool, id: i64) -> Result<Option<Record>> {
    let record = sqlx::query_as::<_, Record>("SELECT * FROM records WHERE id = ?")
        .bind(id)
        .fetch_optional(pool)
        .await
        .context("レコード取得に失敗")?;

    Ok(record)
}

/// レコードを作成
pub async fn create_record(pool: &DbPool, req: CreateRecordRequest) -> Result<i64> {
    let result = sqlx::query(
        "INSERT INTO records (domain_pattern, record_type, content, ttl, active) VALUES (?, ?, ?, ?, 1)"
    )
    .bind(&req.domain_pattern)
    .bind(&req.record_type)
    .bind(&req.content)
    .bind(req.ttl)
    .execute(pool)
    .await
    .context("レコード作成に失敗")?;

    Ok(result.last_insert_rowid())
}

/// レコードを更新
pub async fn update_record(pool: &DbPool, id: i64, req: UpdateRecordRequest) -> Result<bool> {
    // 既存レコードを取得
    let mut record = match get_record_by_id(pool, id).await? {
        Some(r) => r,
        None => return Ok(false),
    };

    // 更新内容を反映
    if let Some(domain_pattern) = req.domain_pattern {
        record.domain_pattern = domain_pattern;
    }
    if let Some(record_type) = req.record_type {
        record.record_type = record_type;
    }
    if let Some(content) = req.content {
        record.content = content;
    }
    if let Some(ttl) = req.ttl {
        record.ttl = ttl;
    }
    if let Some(active) = req.active {
        record.active = active;
    }

    // 更新実行
    sqlx::query(
        "UPDATE records SET domain_pattern = ?, record_type = ?, content = ?, ttl = ?, active = ? WHERE id = ?"
    )
    .bind(&record.domain_pattern)
    .bind(&record.record_type)
    .bind(&record.content)
    .bind(record.ttl)
    .bind(record.active)
    .bind(id)
    .execute(pool)
    .await
    .context("レコード更新に失敗")?;

    Ok(true)
}

/// レコードを削除
pub async fn delete_record(pool: &DbPool, id: i64) -> Result<bool> {
    let result = sqlx::query("DELETE FROM records WHERE id = ?")
        .bind(id)
        .execute(pool)
        .await
        .context("レコード削除に失敗")?;

    Ok(result.rows_affected() > 0)
}

/// クエリログを記録
pub async fn log_query(pool: &DbPool, log: NewQueryLog) -> Result<()> {
    sqlx::query(
        "INSERT INTO query_logs (query_name, q_type, result_type, duration_ms) VALUES (?, ?, ?, ?)"
    )
    .bind(&log.query_name)
    .bind(&log.q_type)
    .bind(&log.result_type)
    .bind(log.duration_ms)
    .execute(pool)
    .await
    .context("クエリログ記録に失敗")?;

    Ok(())
}

/// 最新のクエリログを取得
pub async fn get_recent_logs(pool: &DbPool, limit: i64) -> Result<Vec<QueryLog>> {
    let logs = sqlx::query_as::<_, QueryLog>(
        "SELECT * FROM query_logs ORDER BY timestamp DESC LIMIT ?"
    )
    .bind(limit)
    .fetch_all(pool)
    .await
    .context("クエリログ取得に失敗")?;

    Ok(logs)
}

/// 古いログを削除（将来の定期実行用）
#[allow(dead_code)]
pub async fn cleanup_old_logs(pool: &DbPool, retention_days: i64) -> Result<u64> {
    let result = sqlx::query(
        "DELETE FROM query_logs WHERE timestamp < datetime('now', '-' || ? || ' days')"
    )
    .bind(retention_days)
    .execute(pool)
    .await
    .context("古いログの削除に失敗")?;

    Ok(result.rows_affected())
}

/// 設定を取得
pub async fn get_setting(pool: &DbPool, key: &str) -> Result<Option<String>> {
    let setting = sqlx::query_as::<_, Setting>("SELECT * FROM settings WHERE key = ?")
        .bind(key)
        .fetch_optional(pool)
        .await
        .context("設定取得に失敗")?;

    Ok(setting.map(|s| s.value))
}

/// 全設定を取得
pub async fn get_all_settings(pool: &DbPool) -> Result<Vec<Setting>> {
    let settings = sqlx::query_as::<_, Setting>("SELECT * FROM settings ORDER BY key")
        .fetch_all(pool)
        .await
        .context("設定取得に失敗")?;

    Ok(settings)
}

/// 設定を更新
pub async fn update_setting(pool: &DbPool, key: &str, value: &str) -> Result<()> {
    sqlx::query("INSERT OR REPLACE INTO settings (key, value) VALUES (?, ?)")
        .bind(key)
        .bind(value)
        .execute(pool)
        .await
        .context("設定更新に失敗")?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    async fn setup_test_db() -> DbPool {
        let pool = init_db("sqlite::memory:").await.unwrap();
        pool
    }

    #[tokio::test]
    async fn test_create_and_get_record() {
        let pool = setup_test_db().await;

        let req = CreateRecordRequest {
            domain_pattern: "app.local.test".to_string(),
            record_type: "A".to_string(),
            content: "127.0.0.1".to_string(),
            ttl: 60,
        };

        let id = create_record(&pool, req).await.unwrap();
        assert!(id > 0);

        let record = get_record_by_id(&pool, id).await.unwrap().unwrap();
        assert_eq!(record.domain_pattern, "app.local.test");
        assert_eq!(record.record_type, "A");
        assert_eq!(record.content, "127.0.0.1");
    }

    #[tokio::test]
    async fn test_update_record() {
        let pool = setup_test_db().await;

        let req = CreateRecordRequest {
            domain_pattern: "app.local.test".to_string(),
            record_type: "A".to_string(),
            content: "127.0.0.1".to_string(),
            ttl: 60,
        };

        let id = create_record(&pool, req).await.unwrap();

        let update_req = UpdateRecordRequest {
            domain_pattern: None,
            record_type: None,
            content: Some("192.168.1.1".to_string()),
            ttl: None,
            active: None,
        };

        let updated = update_record(&pool, id, update_req).await.unwrap();
        assert!(updated);

        let record = get_record_by_id(&pool, id).await.unwrap().unwrap();
        assert_eq!(record.content, "192.168.1.1");
    }

    #[tokio::test]
    async fn test_delete_record() {
        let pool = setup_test_db().await;

        let req = CreateRecordRequest {
            domain_pattern: "app.local.test".to_string(),
            record_type: "A".to_string(),
            content: "127.0.0.1".to_string(),
            ttl: 60,
        };

        let id = create_record(&pool, req).await.unwrap();
        let deleted = delete_record(&pool, id).await.unwrap();
        assert!(deleted);

        let record = get_record_by_id(&pool, id).await.unwrap();
        assert!(record.is_none());
    }

    #[tokio::test]
    async fn test_settings() {
        let pool = setup_test_db().await;

        // 初期値確認
        let primary = get_setting(&pool, "upstream_primary").await.unwrap();
        assert_eq!(primary, Some("8.8.8.8:53".to_string()));

        // 更新
        update_setting(&pool, "upstream_primary", "1.1.1.1:53")
            .await
            .unwrap();

        let primary = get_setting(&pool, "upstream_primary").await.unwrap();
        assert_eq!(primary, Some("1.1.1.1:53".to_string()));
    }
}
