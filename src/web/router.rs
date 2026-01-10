use axum::{
    body::Body,
    http::{header, Request, Response, StatusCode},
    routing::get,
    Router,
};
use include_dir::{include_dir, Dir};
use tower_http::cors::{Any, CorsLayer};

/// ビルド済みのフロントエンドファイルを埋め込み
static STATIC_DIR: Dir<'_> = include_dir!("$CARGO_MANIFEST_DIR/web-ui/dist");

/// Webルーターを作成
pub fn create_router(api_router: Router) -> Router {
    // CORSレイヤー（開発用）
    let cors = CorsLayer::new()
        .allow_origin(Any)
        .allow_methods(Any)
        .allow_headers(Any);

    Router::new()
        .merge(api_router)
        // 静的ファイル配信（SPAフォールバック付き）
        .fallback(get(serve_static))
        .layer(cors)
}

/// 静的ファイルを配信（SPAフォールバック対応）
async fn serve_static(req: Request<Body>) -> Response<Body> {
    let path = req.uri().path().trim_start_matches('/');

    // パスが空または拡張子がない場合はindex.htmlを返す（SPAルーティング対応）
    let file_path = if path.is_empty() || !path.contains('.') {
        "index.html"
    } else {
        path
    };

    // ファイルを取得
    if let Some(file) = STATIC_DIR.get_file(file_path) {
        let mime_type = get_mime_type(file_path);
        let body = Body::from(file.contents().to_vec());

        Response::builder()
            .status(StatusCode::OK)
            .header(header::CONTENT_TYPE, mime_type)
            .body(body)
            .unwrap()
    } else if file_path != "index.html" {
        // ファイルが見つからない場合はindex.htmlにフォールバック（SPA対応）
        if let Some(index) = STATIC_DIR.get_file("index.html") {
            let body = Body::from(index.contents().to_vec());

            Response::builder()
                .status(StatusCode::OK)
                .header(header::CONTENT_TYPE, "text/html; charset=utf-8")
                .body(body)
                .unwrap()
        } else {
            not_found_response()
        }
    } else {
        not_found_response()
    }
}

/// MIMEタイプを取得
fn get_mime_type(path: &str) -> &'static str {
    match path.rsplit('.').next() {
        Some("html") => "text/html; charset=utf-8",
        Some("css") => "text/css; charset=utf-8",
        Some("js") => "application/javascript; charset=utf-8",
        Some("json") => "application/json; charset=utf-8",
        Some("png") => "image/png",
        Some("jpg") | Some("jpeg") => "image/jpeg",
        Some("gif") => "image/gif",
        Some("svg") => "image/svg+xml",
        Some("ico") => "image/x-icon",
        Some("woff") => "font/woff",
        Some("woff2") => "font/woff2",
        Some("ttf") => "font/ttf",
        _ => "application/octet-stream",
    }
}

/// 404レスポンスを返す
fn not_found_response() -> Response<Body> {
    Response::builder()
        .status(StatusCode::NOT_FOUND)
        .header(header::CONTENT_TYPE, "text/plain; charset=utf-8")
        .body(Body::from("Not Found"))
        .unwrap()
}
