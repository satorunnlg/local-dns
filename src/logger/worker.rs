use crate::db::{log_query, DbPool, NewQueryLog};
use tokio::sync::mpsc;
use tracing::{debug, error};

/// クエリログメッセージ
#[derive(Debug, Clone)]
pub struct QueryLogMessage {
    pub query_name: String,
    pub q_type: String,
    pub result_type: String,
    pub duration_ms: i64,
}

/// 非同期ログワーカー
pub struct LogWorker {
    sender: mpsc::UnboundedSender<QueryLogMessage>,
}

impl LogWorker {
    /// 新しいログワーカーを作成し、バックグラウンドタスクを起動
    pub fn new(pool: DbPool) -> Self {
        let (sender, receiver) = mpsc::unbounded_channel();

        // バックグラウンドでログ書き込みタスクを起動
        tokio::spawn(async move {
            Self::run_worker(pool, receiver).await;
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
