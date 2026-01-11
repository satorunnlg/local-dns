use crate::db::*;
use crate::dns::RecordCache;
use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::IntoResponse,
    Json, Router,
    routing::{delete, get, post, put},
};
use serde_json::json;
use std::sync::Arc;

/// API状態
#[derive(Clone)]
pub struct ApiState {
    pub pool: DbPool,
    pub cache: RecordCache,
}

/// APIルートを作成
pub fn create_api_routes(state: ApiState) -> Router {
    Router::new()
        // レコード関連
        .route("/api/records", get(get_records))
        .route("/api/records", post(create_record_handler))
        .route("/api/records/:id", get(get_record))
        .route("/api/records/:id", put(update_record_handler))
        .route("/api/records/:id", delete(delete_record_handler))
        // ログ関連
        .route("/api/logs", get(get_logs))
        // 設定関連
        .route("/api/settings", get(get_settings))
        .route("/api/settings/:key", put(update_setting_handler))
        // ヘルスチェック
        .route("/api/health", get(health_check))
        .with_state(Arc::new(state))
}

/// レコード一覧取得
async fn get_records(
    State(state): State<Arc<ApiState>>,
) -> Result<Json<Vec<Record>>, AppError> {
    let records = get_all_records(&state.pool).await?;
    Ok(Json(records))
}

/// レコード取得
async fn get_record(
    State(state): State<Arc<ApiState>>,
    Path(id): Path<i64>,
) -> Result<Json<Record>, AppError> {
    match get_record_by_id(&state.pool, id).await? {
        Some(record) => Ok(Json(record)),
        None => Err(AppError::NotFound),
    }
}

/// レコード作成
async fn create_record_handler(
    State(state): State<Arc<ApiState>>,
    Json(req): Json<CreateRecordRequest>,
) -> Result<Json<serde_json::Value>, AppError> {
    // バリデーション
    validate_record(&req)?;

    let id = create_record(&state.pool, req).await?;

    // キャッシュを再読み込み
    if let Err(e) = state.cache.reload().await {
        tracing::error!("キャッシュ再読み込み失敗: {}", e);
    }

    Ok(Json(json!({ "id": id })))
}

/// レコードのバリデーション
fn validate_record(req: &CreateRecordRequest) -> Result<(), AppError> {
    // ドメインパターンの検証
    if req.domain_pattern.trim().is_empty() {
        return Err(AppError::BadRequest(
            "ドメインパターンを指定してください".to_string(),
        ));
    }

    // レコードタイプの検証
    if !matches!(req.record_type.as_str(), "A" | "AAAA" | "CNAME") {
        return Err(AppError::BadRequest(format!(
            "サポートされていないレコードタイプです: {}",
            req.record_type
        )));
    }

    // コンテンツの検証
    if req.content.trim().is_empty() {
        return Err(AppError::BadRequest(
            "コンテンツを指定してください".to_string(),
        ));
    }

    // レコードタイプごとのコンテンツ検証
    match req.record_type.as_str() {
        "A" => {
            use std::net::Ipv4Addr;
            use std::str::FromStr;
            if Ipv4Addr::from_str(&req.content).is_err() {
                return Err(AppError::BadRequest(
                    "無効なIPv4アドレス形式です".to_string(),
                ));
            }
        }
        "AAAA" => {
            use std::net::Ipv6Addr;
            use std::str::FromStr;
            if Ipv6Addr::from_str(&req.content).is_err() {
                return Err(AppError::BadRequest(
                    "無効なIPv6アドレス形式です".to_string(),
                ));
            }
        }
        "CNAME" => {
            // CNAMEは基本的な文字列チェックのみ
            if req.content.contains(' ') {
                return Err(AppError::BadRequest(
                    "CNAMEに空白文字を含めることはできません".to_string(),
                ));
            }
        }
        _ => {}
    }

    // TTLの検証
    if req.ttl < 1 || req.ttl > 86400 {
        return Err(AppError::BadRequest(
            "TTLは1秒から86400秒(24時間)の範囲で指定してください".to_string(),
        ));
    }

    Ok(())
}

/// レコード更新
async fn update_record_handler(
    State(state): State<Arc<ApiState>>,
    Path(id): Path<i64>,
    Json(req): Json<UpdateRecordRequest>,
) -> Result<StatusCode, AppError> {
    let updated = update_record(&state.pool, id, req).await?;

    if updated {
        // キャッシュを再読み込み
        if let Err(e) = state.cache.reload().await {
            tracing::error!("キャッシュ再読み込み失敗: {}", e);
        }
        Ok(StatusCode::OK)
    } else {
        Err(AppError::NotFound)
    }
}

/// レコード削除
async fn delete_record_handler(
    State(state): State<Arc<ApiState>>,
    Path(id): Path<i64>,
) -> Result<StatusCode, AppError> {
    let deleted = delete_record(&state.pool, id).await?;

    if deleted {
        // キャッシュを再読み込み
        if let Err(e) = state.cache.reload().await {
            tracing::error!("キャッシュ再読み込み失敗: {}", e);
        }
        Ok(StatusCode::OK)
    } else {
        Err(AppError::NotFound)
    }
}

/// ログ一覧取得
async fn get_logs(
    State(state): State<Arc<ApiState>>,
) -> Result<Json<Vec<QueryLog>>, AppError> {
    let logs = get_recent_logs(&state.pool, 100).await?;
    Ok(Json(logs))
}

/// 設定一覧取得
async fn get_settings(
    State(state): State<Arc<ApiState>>,
) -> Result<Json<Vec<Setting>>, AppError> {
    let settings = get_all_settings(&state.pool).await?;
    Ok(Json(settings))
}

/// 設定更新
async fn update_setting_handler(
    State(state): State<Arc<ApiState>>,
    Path(key): Path<String>,
    Json(req): Json<UpdateSettingRequest>,
) -> Result<StatusCode, AppError> {
    update_setting(&state.pool, &key, &req.value).await?;
    Ok(StatusCode::OK)
}

/// ヘルスチェック
async fn health_check() -> Json<serde_json::Value> {
    Json(json!({
        "status": "ok",
        "service": "local-dns-pro"
    }))
}

/// エラーハンドリング
#[derive(Debug)]
enum AppError {
    Internal(anyhow::Error),
    NotFound,
    BadRequest(String),
}

impl From<anyhow::Error> for AppError {
    fn from(err: anyhow::Error) -> Self {
        AppError::Internal(err)
    }
}

impl IntoResponse for AppError {
    fn into_response(self) -> axum::response::Response {
        let (status, message) = match self {
            AppError::Internal(e) => {
                tracing::error!("内部エラー: {:?}", e);
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    "内部サーバーエラーが発生しました".to_string(),
                )
            }
            AppError::NotFound => (
                StatusCode::NOT_FOUND,
                "リソースが見つかりません".to_string(),
            ),
            AppError::BadRequest(msg) => {
                tracing::warn!("不正なリクエスト: {}", msg);
                (StatusCode::BAD_REQUEST, msg)
            }
        };

        (status, Json(json!({ "error": message }))).into_response()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::db::init_db;
    use crate::dns::RecordCache;
    use axum::body::Body;
    use axum::http::Request;
    use http_body_util::BodyExt;
    use tower::ServiceExt;

    /// テスト用のAPIステートを作成
    async fn setup_test_api() -> Router {
        let pool = init_db("sqlite::memory:").await.unwrap();
        let cache = RecordCache::new(pool.clone()).await.unwrap();
        let state = ApiState { pool, cache };
        create_api_routes(state)
    }

    #[tokio::test]
    async fn test_health_check() {
        let app = setup_test_api().await;

        let response = app
            .oneshot(
                Request::builder()
                    .uri("/api/health")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::OK);

        let body = response.into_body().collect().await.unwrap().to_bytes();
        let json: serde_json::Value = serde_json::from_slice(&body).unwrap();
        assert_eq!(json["status"], "ok");
        assert_eq!(json["service"], "local-dns-pro");
    }

    #[tokio::test]
    async fn test_get_records_empty() {
        let app = setup_test_api().await;

        let response = app
            .oneshot(
                Request::builder()
                    .uri("/api/records")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::OK);

        let body = response.into_body().collect().await.unwrap().to_bytes();
        let json: Vec<serde_json::Value> = serde_json::from_slice(&body).unwrap();
        assert!(json.is_empty());
    }

    #[tokio::test]
    async fn test_create_and_get_record() {
        let pool = init_db("sqlite::memory:").await.unwrap();
        let cache = RecordCache::new(pool.clone()).await.unwrap();
        let state = ApiState {
            pool: pool.clone(),
            cache,
        };
        let app = create_api_routes(state);

        // レコード作成
        let create_body = serde_json::json!({
            "domain_pattern": "app.local.test",
            "record_type": "A",
            "content": "192.168.1.100",
            "ttl": 60
        });

        let response = app
            .clone()
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri("/api/records")
                    .header("Content-Type", "application/json")
                    .body(Body::from(create_body.to_string()))
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::OK);

        let body = response.into_body().collect().await.unwrap().to_bytes();
        let json: serde_json::Value = serde_json::from_slice(&body).unwrap();
        let id = json["id"].as_i64().unwrap();
        assert!(id > 0);

        // レコード取得
        let response = app
            .oneshot(
                Request::builder()
                    .uri(format!("/api/records/{}", id))
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::OK);

        let body = response.into_body().collect().await.unwrap().to_bytes();
        let record: serde_json::Value = serde_json::from_slice(&body).unwrap();
        assert_eq!(record["domain_pattern"], "app.local.test");
        assert_eq!(record["record_type"], "A");
        assert_eq!(record["content"], "192.168.1.100");
    }

    #[tokio::test]
    async fn test_create_record_validation_empty_domain() {
        let app = setup_test_api().await;

        let create_body = serde_json::json!({
            "domain_pattern": "",
            "record_type": "A",
            "content": "192.168.1.100",
            "ttl": 60,
                    });

        let response = app
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri("/api/records")
                    .header("Content-Type", "application/json")
                    .body(Body::from(create_body.to_string()))
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::BAD_REQUEST);
    }

    #[tokio::test]
    async fn test_create_record_validation_invalid_ip() {
        let app = setup_test_api().await;

        let create_body = serde_json::json!({
            "domain_pattern": "app.local.test",
            "record_type": "A",
            "content": "invalid-ip",
            "ttl": 60,
                    });

        let response = app
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri("/api/records")
                    .header("Content-Type", "application/json")
                    .body(Body::from(create_body.to_string()))
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::BAD_REQUEST);
    }

    #[tokio::test]
    async fn test_create_record_validation_invalid_type() {
        let app = setup_test_api().await;

        let create_body = serde_json::json!({
            "domain_pattern": "app.local.test",
            "record_type": "MX",
            "content": "mail.example.com",
            "ttl": 60,
                    });

        let response = app
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri("/api/records")
                    .header("Content-Type", "application/json")
                    .body(Body::from(create_body.to_string()))
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::BAD_REQUEST);
    }

    #[tokio::test]
    async fn test_create_record_validation_invalid_ttl() {
        let app = setup_test_api().await;

        let create_body = serde_json::json!({
            "domain_pattern": "app.local.test",
            "record_type": "A",
            "content": "192.168.1.100",
            "ttl": 0,
                    });

        let response = app
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri("/api/records")
                    .header("Content-Type", "application/json")
                    .body(Body::from(create_body.to_string()))
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::BAD_REQUEST);
    }

    #[tokio::test]
    async fn test_get_record_not_found() {
        let app = setup_test_api().await;

        let response = app
            .oneshot(
                Request::builder()
                    .uri("/api/records/99999")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::NOT_FOUND);
    }

    #[tokio::test]
    async fn test_delete_record() {
        let pool = init_db("sqlite::memory:").await.unwrap();
        let cache = RecordCache::new(pool.clone()).await.unwrap();
        let state = ApiState {
            pool: pool.clone(),
            cache,
        };
        let app = create_api_routes(state);

        // レコード作成
        let create_body = serde_json::json!({
            "domain_pattern": "delete.local.test",
            "record_type": "A",
            "content": "10.0.0.1",
            "ttl": 60
        });

        let response = app
            .clone()
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri("/api/records")
                    .header("Content-Type", "application/json")
                    .body(Body::from(create_body.to_string()))
                    .unwrap(),
            )
            .await
            .unwrap();

        let body = response.into_body().collect().await.unwrap().to_bytes();
        let json: serde_json::Value = serde_json::from_slice(&body).unwrap();
        let id = json["id"].as_i64().unwrap();

        // レコード削除
        let response = app
            .clone()
            .oneshot(
                Request::builder()
                    .method("DELETE")
                    .uri(format!("/api/records/{}", id))
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::OK);

        // 削除後に取得 → NotFound
        let response = app
            .oneshot(
                Request::builder()
                    .uri(format!("/api/records/{}", id))
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::NOT_FOUND);
    }

    #[tokio::test]
    async fn test_get_settings() {
        let app = setup_test_api().await;

        let response = app
            .oneshot(
                Request::builder()
                    .uri("/api/settings")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::OK);

        let body = response.into_body().collect().await.unwrap().to_bytes();
        let settings: Vec<serde_json::Value> = serde_json::from_slice(&body).unwrap();

        // 初期設定が存在することを確認
        let keys: Vec<&str> = settings
            .iter()
            .filter_map(|s| s["key"].as_str())
            .collect();
        assert!(keys.contains(&"upstream_primary"));
        assert!(keys.contains(&"upstream_secondary"));
    }

    #[tokio::test]
    async fn test_update_setting() {
        let pool = init_db("sqlite::memory:").await.unwrap();
        let cache = RecordCache::new(pool.clone()).await.unwrap();
        let state = ApiState {
            pool: pool.clone(),
            cache,
        };
        let app = create_api_routes(state);

        // 設定更新
        let update_body = serde_json::json!({
            "value": "9.9.9.9:53"
        });

        let response = app
            .clone()
            .oneshot(
                Request::builder()
                    .method("PUT")
                    .uri("/api/settings/upstream_primary")
                    .header("Content-Type", "application/json")
                    .body(Body::from(update_body.to_string()))
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::OK);

        // 更新後の設定を確認
        let response = app
            .oneshot(
                Request::builder()
                    .uri("/api/settings")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        let body = response.into_body().collect().await.unwrap().to_bytes();
        let settings: Vec<serde_json::Value> = serde_json::from_slice(&body).unwrap();

        let primary = settings
            .iter()
            .find(|s| s["key"] == "upstream_primary")
            .unwrap();
        assert_eq!(primary["value"], "9.9.9.9:53");
    }

    #[tokio::test]
    async fn test_get_logs_empty() {
        let app = setup_test_api().await;

        let response = app
            .oneshot(
                Request::builder()
                    .uri("/api/logs")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::OK);

        let body = response.into_body().collect().await.unwrap().to_bytes();
        let logs: Vec<serde_json::Value> = serde_json::from_slice(&body).unwrap();
        assert!(logs.is_empty());
    }

    #[tokio::test]
    async fn test_validate_record_ipv6() {
        // 有効なIPv6
        let req = CreateRecordRequest {
            domain_pattern: "app.local.test".to_string(),
            record_type: "AAAA".to_string(),
            content: "::1".to_string(),
            ttl: 60,
        };
        assert!(validate_record(&req).is_ok());

        // 無効なIPv6
        let req = CreateRecordRequest {
            domain_pattern: "app.local.test".to_string(),
            record_type: "AAAA".to_string(),
            content: "invalid-ipv6".to_string(),
            ttl: 60,
        };
        assert!(validate_record(&req).is_err());
    }

    #[tokio::test]
    async fn test_validate_record_cname() {
        // 有効なCNAME
        let req = CreateRecordRequest {
            domain_pattern: "alias.local.test".to_string(),
            record_type: "CNAME".to_string(),
            content: "target.local.test".to_string(),
            ttl: 60,
        };
        assert!(validate_record(&req).is_ok());

        // 空白を含むCNAME（無効）
        let req = CreateRecordRequest {
            domain_pattern: "alias.local.test".to_string(),
            record_type: "CNAME".to_string(),
            content: "invalid target".to_string(),
            ttl: 60,
        };
        assert!(validate_record(&req).is_err());
    }

    #[tokio::test]
    async fn test_validate_record_empty_content() {
        let req = CreateRecordRequest {
            domain_pattern: "app.local.test".to_string(),
            record_type: "A".to_string(),
            content: "   ".to_string(),
            ttl: 60,
        };
        assert!(validate_record(&req).is_err());
    }

    #[tokio::test]
    async fn test_validate_record_ttl_too_high() {
        let req = CreateRecordRequest {
            domain_pattern: "app.local.test".to_string(),
            record_type: "A".to_string(),
            content: "192.168.1.1".to_string(),
            ttl: 100000,
        };
        assert!(validate_record(&req).is_err());
    }
}
