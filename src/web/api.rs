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
    let id = create_record(&state.pool, req).await?;

    // キャッシュを再読み込み
    if let Err(e) = state.cache.reload().await {
        tracing::error!("キャッシュ再読み込み失敗: {}", e);
    }

    Ok(Json(json!({ "id": id })))
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
                tracing::error!("Internal error: {}", e);
                (StatusCode::INTERNAL_SERVER_ERROR, "Internal server error")
            }
            AppError::NotFound => (StatusCode::NOT_FOUND, "Not found"),
        };

        (status, Json(json!({ "error": message }))).into_response()
    }
}
