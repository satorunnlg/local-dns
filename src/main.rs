mod db;
mod dns;
mod logger;
mod web;

use anyhow::{Context, Result};
use db::init_db;
use dns::{RecordCache, UpstreamConfig};
use logger::LogWorker;
use std::net::SocketAddr;
use tracing::{error, info};
use tracing_subscriber::EnvFilter;
use web::{api::ApiState, create_api_routes, create_router};

#[tokio::main]
async fn main() {
    // ロギング初期化
    tracing_subscriber::fmt()
        .with_env_filter(
            EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| EnvFilter::new("info")),
        )
        .init();

    info!("LocalDNS Pro 起動中...");

    if let Err(e) = run().await {
        error!("エラー: {:?}", e);
        std::process::exit(1);
    }
}

async fn run() -> Result<()> {
    // データベース初期化
    let pool = init_db("sqlite:dns.db")
        .await
        .context("データベース初期化に失敗")?;

    info!("データベース初期化完了");

    // レコードキャッシュ初期化
    let cache = RecordCache::new(pool.clone())
        .await
        .context("レコードキャッシュ初期化に失敗")?;

    info!("レコードキャッシュ初期化完了");

    // ログワーカー起動
    let _log_worker = LogWorker::new(pool.clone());
    info!("ログワーカー起動完了");

    // 上位DNS設定取得
    let primary = db::get_setting(&pool, "upstream_primary")
        .await?
        .unwrap_or_else(|| "8.8.8.8:53".to_string());

    let secondary = db::get_setting(&pool, "upstream_secondary")
        .await?
        .unwrap_or_else(|| "1.1.1.1:53".to_string());

    let timeout_ms = db::get_setting(&pool, "upstream_timeout_ms")
        .await?
        .and_then(|s| s.parse().ok())
        .unwrap_or(2000);

    let _upstream_config = UpstreamConfig::new(&primary, &secondary, timeout_ms)
        .context("上位DNS設定の初期化に失敗")?;

    info!(
        "上位DNS設定: Primary={}, Secondary={}, Timeout={}ms",
        primary, secondary, timeout_ms
    );

    // Web API状態
    let api_state = ApiState {
        pool: pool.clone(),
        cache: cache.clone(),
    };

    // Webルーター作成
    let api_router = create_api_routes(api_state);
    let app = create_router(api_router);

    // Webサーバー起動
    let web_addr = SocketAddr::from(([0, 0, 0, 0], 3000));
    info!("Web UI起動: http://{}", web_addr);

    let listener = tokio::net::TcpListener::bind(web_addr)
        .await
        .context("Webサーバーのバインドに失敗")?;

    axum::serve(listener, app)
        .await
        .context("Webサーバーの実行に失敗")?;

    Ok(())
}
