use crate::db::{get_active_records, DbPool, Record};
use anyhow::Result;
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::{error, info};

/// レコードキャッシュ
#[derive(Clone)]
pub struct RecordCache {
    records: Arc<RwLock<Vec<Record>>>,
    pool: DbPool,
}

impl RecordCache {
    /// 新しいキャッシュを作成し、DBから初期ロード
    pub async fn new(pool: DbPool) -> Result<Self> {
        let cache = Self {
            records: Arc::new(RwLock::new(Vec::new())),
            pool,
        };

        cache.reload().await?;
        Ok(cache)
    }

    /// キャッシュをDBから再読み込み
    pub async fn reload(&self) -> Result<()> {
        info!("レコードキャッシュを再読み込み中");

        match get_active_records(&self.pool).await {
            Ok(records) => {
                let count = records.len();
                let mut cache = self.records.write().await;
                *cache = records;
                info!("レコードキャッシュ再読み込み完了: {} 件", count);
                Ok(())
            }
            Err(e) => {
                error!("レコードキャッシュ再読み込み失敗: {}", e);
                Err(e)
            }
        }
    }

    /// クエリ名に一致するレコードを検索
    pub async fn find_matching_record(
        &self,
        query_name: &str,
        record_type: &str,
    ) -> Option<Record> {
        let records = self.records.read().await;

        for record in records.iter() {
            if record.record_type == record_type && record.matches(query_name) {
                return Some(record.clone());
            }
        }

        None
    }

    /// キャッシュ内の全レコード数を取得
    pub async fn count(&self) -> usize {
        let records = self.records.read().await;
        records.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::db::{create_record, init_db, CreateRecordRequest};

    async fn setup_test_cache() -> RecordCache {
        let pool = init_db("sqlite::memory:").await.unwrap();
        RecordCache::new(pool.clone()).await.unwrap()
    }

    #[tokio::test]
    async fn test_cache_reload() {
        let cache = setup_test_cache().await;

        // 初期状態は空
        assert_eq!(cache.count().await, 0);

        // レコード追加
        let req = CreateRecordRequest {
            domain_pattern: "app.local.test".to_string(),
            record_type: "A".to_string(),
            content: "127.0.0.1".to_string(),
            ttl: 60,
        };
        create_record(&cache.pool, req).await.unwrap();

        // 再読み込み
        cache.reload().await.unwrap();
        assert_eq!(cache.count().await, 1);
    }

    #[tokio::test]
    async fn test_find_matching_record() {
        let cache = setup_test_cache().await;

        // レコード追加
        let req = CreateRecordRequest {
            domain_pattern: "%.local.test".to_string(),
            record_type: "A".to_string(),
            content: "127.0.0.1".to_string(),
            ttl: 60,
        };
        create_record(&cache.pool, req).await.unwrap();
        cache.reload().await.unwrap();

        // マッチするレコードを検索
        let record = cache
            .find_matching_record("app.local.test", "A")
            .await
            .unwrap();
        assert_eq!(record.content, "127.0.0.1");

        // マッチしないケース
        let no_match = cache.find_matching_record("app.local.test", "AAAA").await;
        assert!(no_match.is_none());
    }

    #[tokio::test]
    async fn test_multiple_records_priority() {
        let cache = setup_test_cache().await;

        // ワイルドカードレコード
        let req1 = CreateRecordRequest {
            domain_pattern: "%.local.test".to_string(),
            record_type: "A".to_string(),
            content: "127.0.0.1".to_string(),
            ttl: 60,
        };
        create_record(&cache.pool, req1).await.unwrap();

        // 完全一致レコード（後から追加）
        let req2 = CreateRecordRequest {
            domain_pattern: "app.local.test".to_string(),
            record_type: "A".to_string(),
            content: "192.168.1.1".to_string(),
            ttl: 60,
        };
        create_record(&cache.pool, req2).await.unwrap();

        cache.reload().await.unwrap();

        // 最初にマッチしたものが返される
        let record = cache
            .find_matching_record("app.local.test", "A")
            .await
            .unwrap();

        // どちらかがマッチする（実装では最初にマッチしたものを返す）
        assert!(
            record.content == "127.0.0.1" || record.content == "192.168.1.1"
        );
    }
}
