use anyhow::{Context, Result};
use hickory_proto::op::Query;
use hickory_proto::rr::{Name, RecordType};
use std::net::SocketAddr;
use std::str::FromStr;
use std::time::Duration;
use tracing::{debug, warn};

/// 上位DNS設定
#[derive(Clone, Debug)]
pub struct UpstreamConfig {
    pub primary: SocketAddr,
    pub secondary: SocketAddr,
    pub timeout: Duration,
}

impl UpstreamConfig {
    /// 設定値から作成
    pub fn new(
        primary: &str,
        secondary: &str,
        timeout_ms: u64,
    ) -> Result<Self> {
        let primary = SocketAddr::from_str(primary)
            .context(format!("Primary DNS アドレスのパースに失敗: {}", primary))?;

        let secondary = SocketAddr::from_str(secondary)
            .context(format!("Secondary DNS アドレスのパースに失敗: {}", secondary))?;

        Ok(Self {
            primary,
            secondary,
            timeout: Duration::from_millis(timeout_ms),
        })
    }
}

/// 上位DNSクライアント
pub struct UpstreamResolver {
    config: UpstreamConfig,
}

impl UpstreamResolver {
    pub fn new(config: UpstreamConfig) -> Self {
        Self { config }
    }

    /// 上位DNSに問い合わせ
    pub async fn query(
        &self,
        query_name: &str,
        record_type: &str,
    ) -> Result<Vec<hickory_proto::rr::Record>> {
        debug!(
            "上位DNS問い合わせ: {} ({})",
            query_name, record_type
        );

        // レコードタイプをパース
        let rtype = match record_type {
            "A" => RecordType::A,
            "AAAA" => RecordType::AAAA,
            "CNAME" => RecordType::CNAME,
            _ => {
                warn!("サポートされていないレコードタイプ: {}", record_type);
                return Ok(vec![]);
            }
        };

        // ドメイン名をパース
        let name = Name::from_str(query_name)
            .context(format!("ドメイン名のパースに失敗: {}", query_name))?;

        // まずプライマリDNSに問い合わせ
        match self.query_upstream(self.config.primary, &name, rtype).await {
            Ok(records) => {
                debug!("プライマリDNSから応答を取得: {} レコード", records.len());
                return Ok(records);
            }
            Err(e) => {
                warn!("プライマリDNSへの問い合わせ失敗: {}", e);
            }
        }

        // プライマリが失敗した場合、セカンダリDNSに問い合わせ
        match self.query_upstream(self.config.secondary, &name, rtype).await {
            Ok(records) => {
                debug!("セカンダリDNSから応答を取得: {} レコード", records.len());
                Ok(records)
            }
            Err(e) => {
                warn!("セカンダリDNSへの問い合わせ失敗: {}", e);
                Err(e)
            }
        }
    }

    /// 指定した上位DNSに問い合わせ
    async fn query_upstream(
        &self,
        server: SocketAddr,
        name: &Name,
        rtype: RecordType,
    ) -> Result<Vec<hickory_proto::rr::Record>> {
        use hickory_proto::op::{Message, MessageType};
        use hickory_proto::serialize::binary::BinDecodable;
        use tokio::net::UdpSocket;

        // UDPソケットを作成
        let socket = UdpSocket::bind("0.0.0.0:0").await?;
        socket.connect(server).await?;

        // DNS問い合わせメッセージを作成
        let mut message = Message::new();
        let id = rand::random::<u16>();
        message.set_id(id);
        message.set_message_type(MessageType::Query);
        message.set_op_code(hickory_proto::op::OpCode::Query);
        message.set_recursion_desired(true);

        let query = Query::query(name.clone(), rtype);
        message.add_query(query);

        // メッセージをバイト列にエンコード
        let request_bytes = message.to_vec()?;

        // タイムアウト付きで送受信
        let result = tokio::time::timeout(
            self.config.timeout,
            async {
                // リクエスト送信
                socket.send(&request_bytes).await?;

                // レスポンス受信（EDNSの最大サイズを考慮して4096バイト）
                let mut response_bytes = vec![0u8; 4096];
                let len = socket.recv(&mut response_bytes).await?;
                response_bytes.truncate(len);

                // レスポンスをデコード
                let response = Message::from_bytes(&response_bytes)?;
                Ok::<Message, anyhow::Error>(response)
            }
        )
        .await
        .context("上位DNSへの問い合わせがタイムアウト")??;

        // レスポンスから答えを抽出
        Ok(result.answers().to_vec())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_upstream_config_new() {
        let config = UpstreamConfig::new("8.8.8.8:53", "1.1.1.1:53", 2000).unwrap();

        assert_eq!(
            config.primary,
            SocketAddr::from_str("8.8.8.8:53").unwrap()
        );
        assert_eq!(
            config.secondary,
            SocketAddr::from_str("1.1.1.1:53").unwrap()
        );
        assert_eq!(config.timeout, Duration::from_millis(2000));
    }

    #[test]
    fn test_upstream_config_invalid_address() {
        let result = UpstreamConfig::new("invalid", "1.1.1.1:53", 2000);
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_query_real() {
        let config = UpstreamConfig::new("8.8.8.8:53", "1.1.1.1:53", 5000).unwrap();
        let resolver = UpstreamResolver::new(config);

        // 実際のDNS問い合わせテスト (google.com は確実に存在する)
        let result = resolver.query("google.com", "A").await;

        // ネットワーク接続がある環境ではOK、ない場合はスキップ
        if result.is_ok() {
            let records = result.unwrap();
            assert!(!records.is_empty(), "google.com の A レコードが取得できませんでした");
        }
    }
}
