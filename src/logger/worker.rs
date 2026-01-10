use crate::db::{cleanup_old_logs, get_setting, log_query, DbPool, NewQueryLog};
use std::time::Duration;
use tokio::sync::mpsc;
use tracing::{debug, error, info};

/// クエリログメッセージ
#[derive(Debug, Clone)]
pub struct QueryLogMessage {
    pub query_name: String,
    pub q_type: String,
    pub result_type: String,
    pub duration_ms: i64,
}

/// ログクリーンアップのデフォルト間隔（1時間）
const CLEANUP_INTERVAL_SECS: u64 = 3600;

/// 非同期ログワーカー
pub struct LogWorker {
    sender: mpsc::UnboundedSender<QueryLogMessage>,
}

impl LogWorker {
    /// 新しいログワーカーを作成し、バックグラウンドタスクを起動
    pub fn new(pool: DbPool) -> Self {
        let (sender, receiver) = mpsc::unbounded_channel();

        // バックグラウンドでログ書き込みタスクを起動
        let pool_for_writer = pool.clone();
        tokio::spawn(async move {
            Self::run_worker(pool_for_writer, receiver).await;
        });

        // バックグラウンドでログクリーンアップタスクを起動
        tokio::spawn(async move {
            Self::run_cleanup_worker(pool).await;
        });

        Self { sender }
    }

    /// ログメッセージを送信
    pub fn log(&self, message: QueryLogMessage) {
        if let Err(e) = self.sender.send(message) {
            error!("ログメッセージの送信に失敗: {}", e);
        }
    }

    /// バックグラウンドでログを書き込み続ける
    async fn run_worker(
        pool: DbPool,
        mut receiver: mpsc::UnboundedReceiver<QueryLogMessage>,
    ) {
        debug!("ログワーカー起動");

        while let Some(message) = receiver.recv().await {
            let log = NewQueryLog {
                query_name: message.query_name,
                q_type: message.q_type,
                result_type: message.result_type,
                duration_ms: message.duration_ms,
            };

            if let Err(e) = log_query(&pool, log).await {
                error!("クエリログの記録に失敗: {}", e);
            } else {
                debug!("クエリログ記録完了");
            }
        }

        debug!("ログワーカー終了");
    }

    /// 定期的に古いログをクリーンアップ
    async fn run_cleanup_worker(pool: DbPool) {
        info!("ログクリーンアップワーカー起動");

        loop {
            // 設定から保存期間を取得
            let retention_days = match get_setting(&pool, "log_retention_days").await {
                Ok(Some(value)) => value.parse().unwrap_or(7),
                _ => 7, // デフォルト7日
            };

            // 古いログを削除
            match cleanup_old_logs(&pool, retention_days).await {
                Ok(deleted_count) => {
                    if deleted_count > 0 {
                        info!(
                            "古いログをクリーンアップ: {} 件削除 (保存期間: {} 日)",
                            deleted_count, retention_days
                        );
                    } else {
                        debug!("クリーンアップ対象のログなし");
                    }
                }
                Err(e) => {
                    error!("ログクリーンアップ失敗: {}", e);
                }
            }

            // 次のクリーンアップまで待機
            tokio::time::sleep(Duration::from_secs(CLEANUP_INTERVAL_SECS)).await;
        }
    }
}

impl Clone for LogWorker {
    fn clone(&self) -> Self {
        Self {
            sender: self.sender.clone(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::db::{get_recent_logs, init_db};

    #[tokio::test]
    async fn test_log_worker() {
        let pool = init_db("sqlite::memory:").await.unwrap();
        let worker = LogWorker::new(pool.clone());

        // ログメッセージを送信
        worker.log(QueryLogMessage {
            query_name: "test.local".to_string(),
            q_type: "A".to_string(),
            result_type: "LOCAL".to_string(),
            duration_ms: 5,
        });

        // 少し待機してログが書き込まれるまで待つ
        tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;

        // ログが記録されているか確認
        let logs = get_recent_logs(&pool, 10).await.unwrap();
        assert_eq!(logs.len(), 1);
        assert_eq!(logs[0].query_name, "test.local");
        assert_eq!(logs[0].q_type, "A");
    }

    #[tokio::test]
    async fn test_log_worker_multiple_messages() {
        let pool = init_db("sqlite::memory:").await.unwrap();
        let worker = LogWorker::new(pool.clone());

        // 複数のログメッセージを送信
        for i in 0..5 {
            worker.log(QueryLogMessage {
                query_name: format!("test{}.local", i),
                q_type: "A".to_string(),
                result_type: "LOCAL".to_string(),
                duration_ms: i,
            });
        }

        // 少し待機
        tokio::time::sleep(tokio::time::Duration::from_millis(200)).await;

        // ログが記録されているか確認
        let logs = get_recent_logs(&pool, 10).await.unwrap();
        assert_eq!(logs.len(), 5);
    }
}
