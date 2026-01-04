use axum::Router;
use tower_http::cors::{Any, CorsLayer};

/// Webルーターを作成
pub fn create_router(api_router: Router) -> Router {
    // CORSレイヤー（開発用）
    let cors = CorsLayer::new()
        .allow_origin(Any)
        .allow_methods(Any)
        .allow_headers(Any);

    Router::new()
        .merge(api_router)
        .layer(cors)
}
