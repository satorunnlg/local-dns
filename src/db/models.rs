use serde::{Deserialize, Serialize};
use sqlx::FromRow;

/// DNSレコード
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct Record {
    pub id: i64,
    pub domain_pattern: String,
    pub record_type: String,
    pub content: String,
    pub ttl: i64,
    pub active: i64,
}

impl Record {
    /// ドメインパターンがクエリ名にマッチするか判定
    /// SQLiteの LIKE パターンを使用（% はワイルドカード）
    pub fn matches(&self, query_name: &str) -> bool {
        if !self.is_active() {
            return false;
        }

        // パターンをRust正規表現に変換
        let pattern = self.domain_pattern
            .replace('.', r"\.")
            .replace('%', ".*");

        if let Ok(re) = regex::Regex::new(&format!("^{}$", pattern)) {
            re.is_match(query_name)
        } else {
            false
        }
    }

    /// レコードが有効かどうか
    pub fn is_active(&self) -> bool {
        self.active == 1
    }
}

/// クエリログ
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct QueryLog {
    pub id: i64,
    pub query_name: String,
    pub q_type: String,
    pub result_type: String,
    pub duration_ms: i64,
    pub timestamp: String,
}

/// 新規クエリログの作成用
#[derive(Debug, Clone)]
pub struct NewQueryLog {
    pub query_name: String,
    pub q_type: String,
    pub result_type: String,
    pub duration_ms: i64,
}

/// 設定
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct Setting {
    pub key: String,
    pub value: String,
}

/// レコード作成用リクエスト
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateRecordRequest {
    pub domain_pattern: String,
    pub record_type: String,
    pub content: String,
    #[serde(default = "default_ttl")]
    pub ttl: i64,
}

fn default_ttl() -> i64 {
    60
}

/// レコード更新用リクエスト
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateRecordRequest {
    pub domain_pattern: Option<String>,
    pub record_type: Option<String>,
    pub content: Option<String>,
    pub ttl: Option<i64>,
    pub active: Option<i64>,
}

/// 設定更新用リクエスト
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateSettingRequest {
    pub value: String,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_record_matches_exact() {
        let record = Record {
            id: 1,
            domain_pattern: "app.local.test".to_string(),
            record_type: "A".to_string(),
            content: "127.0.0.1".to_string(),
            ttl: 60,
            active: 1,
        };

        assert!(record.matches("app.local.test"));
        assert!(!record.matches("other.local.test"));
    }

    #[test]
    fn test_record_matches_wildcard() {
        let record = Record {
            id: 1,
            domain_pattern: "%.local.test".to_string(),
            record_type: "A".to_string(),
            content: "127.0.0.1".to_string(),
            ttl: 60,
            active: 1,
        };

        assert!(record.matches("app.local.test"));
        assert!(record.matches("api.local.test"));
        assert!(record.matches("anything.local.test"));
        assert!(!record.matches("local.test"));
    }

    #[test]
    fn test_record_matches_inactive() {
        let record = Record {
            id: 1,
            domain_pattern: "app.local.test".to_string(),
            record_type: "A".to_string(),
            content: "127.0.0.1".to_string(),
            ttl: 60,
            active: 0,
        };

        assert!(!record.matches("app.local.test"));
    }
}
