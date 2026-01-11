use crate::dns::{build_dns_record, upstream::UpstreamResolver, RecordCache};
use crate::logger::worker::{LogWorker, QueryLogMessage};
use hickory_server::authority::MessageResponseBuilder;
use hickory_server::proto::op::{Header, MessageType, OpCode, ResponseCode};
use hickory_server::proto::rr::Record as DnsRecord;
use hickory_server::server::{Request, RequestHandler, ResponseHandler, ResponseInfo};
use std::sync::Arc;
use std::time::Instant;
use tracing::{debug, warn};

/// DNSリクエストハンドラ
#[derive(Clone)]
pub struct DnsHandler {
    cache: RecordCache,
    log_worker: LogWorker,
    upstream: Option<Arc<UpstreamResolver>>,
}

impl DnsHandler {
    pub fn new(cache: RecordCache, log_worker: LogWorker) -> Self {
        Self {
            cache,
            log_worker,
            upstream: None,
        }
    }

    /// 上位DNS転送を有効化
    pub fn with_upstream(mut self, upstream: UpstreamResolver) -> Self {
        self.upstream = Some(Arc::new(upstream));
        self
    }

    /// DNS問い合わせを処理
    async fn handle_query(&self, request: &Request) -> Vec<DnsRecord> {
        let start = Instant::now();

        // リクエストから問い合わせ情報を取得
        let request_info = match request.request_info() {
            Ok(info) => info,
            Err(e) => {
                warn!("リクエスト情報の取得に失敗: {}", e);
                return Vec::new();
            }
        };

        let query = request_info.query;
        let query_name_raw = query.name().to_string();
        // 末尾のドットを削除（FQDN表記を正規化）
        let query_name = query_name_raw.trim_end_matches('.').to_string();
        let record_type = query.query_type();

        debug!(
            "DNS問い合わせ受信: {} {:?}",
            query_name, record_type
        );

        let mut answers = Vec::new();
        let mut result_type = "ERROR";

        // キャッシュ検索
        let record_type_str = format!("{:?}", record_type);
        if let Some(db_record) = self
            .cache
            .find_matching_record(&query_name, &record_type_str)
            .await
        {
            debug!(
                "キャッシュヒット: {} -> {}",
                query_name, db_record.content
            );

            if let Some(dns_record) = build_dns_record(query.name(), &db_record) {
                answers.push(dns_record);
                result_type = "LOCAL";
            }
        } else {
            debug!("キャッシュミス: {}", query_name);

            // 上位DNSに転送
            if let Some(upstream) = &self.upstream {
                match upstream.query(&query_name, &record_type_str).await {
                    Ok(records) => {
                        if !records.is_empty() {
                            debug!("上位DNSから {} レコードを取得", records.len());
                            answers.extend(records);
                            result_type = "FORWARDED";
                        }
                    }
                    Err(e) => {
                        warn!("上位DNS問い合わせエラー: {}", e);
                    }
                }
            }
        }

        // ログ記録
        let duration_ms = start.elapsed().as_millis() as i64;
        self.log_worker.log(QueryLogMessage {
            query_name,
            q_type: record_type_str,
            result_type: result_type.to_string(),
            duration_ms,
        });

        answers
    }
}

#[async_trait::async_trait]
impl RequestHandler for DnsHandler {
    async fn handle_request<R: ResponseHandler>(
        &self,
        request: &Request,
        mut response_handle: R,
    ) -> ResponseInfo {
        // ヘッダー取得
        let mut header = Header::response_from_request(request.header());

        // クエリタイプチェック
        if request.op_code() != OpCode::Query {
            header.set_response_code(ResponseCode::NotImp);
            let response = MessageResponseBuilder::from_message_request(request)
                .build_no_records(header);
            return response_handle.send_response(response).await.unwrap();
        }

        // クエリ処理
        let answers = self.handle_query(request).await;

        // レスポンス構築
        header.set_response_code(if answers.is_empty() {
            ResponseCode::NXDomain
        } else {
            ResponseCode::NoError
        });

        let response = MessageResponseBuilder::from_message_request(request)
            .build(header, answers.iter(), &[], &[], &[]);

        match response_handle.send_response(response).await {
            Ok(info) => info,
            Err(e) => {
                warn!("レスポンス送信失敗: {}", e);
                let mut header = Header::new();
                header.set_message_type(MessageType::Response);
                header.set_response_code(ResponseCode::ServFail);
                ResponseInfo::from(header)
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::db::{create_record, init_db, CreateRecordRequest};
    use crate::dns::upstream::UpstreamConfig;

    #[tokio::test]
    async fn test_dns_handler_cache_hit() {
        let pool = init_db("sqlite::memory:").await.unwrap();

        // テストレコード追加
        create_record(
            &pool,
            CreateRecordRequest {
                domain_pattern: "test.local".to_string(),
                record_type: "A".to_string(),
                content: "127.0.0.1".to_string(),
                ttl: 60,
            },
        )
        .await
        .unwrap();

        let cache = RecordCache::new(pool.clone()).await.unwrap();
        let log_worker = LogWorker::new(pool.clone());
        let _handler = DnsHandler::new(cache, log_worker);

        // 実際のDNS問い合わせテストは統合テストで実施
    }

    #[tokio::test]
    async fn test_dns_handler_with_upstream() {
        let pool = init_db("sqlite::memory:").await.unwrap();
        let cache = RecordCache::new(pool.clone()).await.unwrap();
        let log_worker = LogWorker::new(pool.clone());

        let config = UpstreamConfig::new("8.8.8.8:53", "1.1.1.1:53", 2000).unwrap();
        let upstream = UpstreamResolver::new(config);

        let handler = DnsHandler::new(cache, log_worker).with_upstream(upstream);

        // 上位転送が設定されていることを確認
        assert!(handler.upstream.is_some());
    }

    #[tokio::test]
    async fn test_dns_handler_without_upstream() {
        let pool = init_db("sqlite::memory:").await.unwrap();
        let cache = RecordCache::new(pool.clone()).await.unwrap();
        let log_worker = LogWorker::new(pool.clone());

        let handler = DnsHandler::new(cache, log_worker);

        // 上位転送が設定されていないことを確認
        assert!(handler.upstream.is_none());
    }

    #[tokio::test]
    async fn test_dns_handler_clone() {
        let pool = init_db("sqlite::memory:").await.unwrap();
        let cache = RecordCache::new(pool.clone()).await.unwrap();
        let log_worker = LogWorker::new(pool.clone());

        let handler = DnsHandler::new(cache, log_worker);
        let cloned = handler.clone();

        // クローンが正常に動作することを確認
        assert!(cloned.upstream.is_none());
    }
}
