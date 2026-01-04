use anyhow::{Context, Result};
use std::net::SocketAddr;
use std::str::FromStr;
use std::time::Duration;
use tracing::debug;

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

    /// 上位DNSに問い合わせ（スタブ実装）
    /// 注: 完全な実装は将来のバージョンで追加予定
    pub async fn query(
        &self,
        query_name: &str,
        _record_type: &str,
    ) -> Result<Vec<u8>> {
        debug!(
            "上位DNS問い合わせ (スタブ): {}",
            query_name
        );

        // スタブ実装: 空のレスポンスを返す
        // 実際の実装では hickory-client を使用して上位DNSに問い合わせる
        Ok(vec![])
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
    async fn test_query_stub() {
        let config = UpstreamConfig::new("8.8.8.8:53", "1.1.1.1:53", 5000).unwrap();
        let resolver = UpstreamResolver::new(config);

        let result = resolver.query("google.com", "A").await;
        assert!(result.is_ok());
    }
}
