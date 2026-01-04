mod db;
mod dns;
mod logger;
mod web;

use anyhow::{Context, Result};
use db::init_db;
use dns::{upstream::UpstreamResolver, DnsHandler, RecordCache, UpstreamConfig};
use hickory_server::ServerFuture;
use logger::LogWorker;
use std::net::SocketAddr;
use tokio::net::{TcpListener as TokioTcpListener, UdpSocket};
use tokio::signal;
use tracing::{error, info, warn};
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
    let log_worker = LogWorker::new(pool.clone());
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

    let upstream_config = UpstreamConfig::new(&primary, &secondary, timeout_ms)
        .context("上位DNS設定の初期化に失敗")?;

    info!(
        "上位DNS設定: Primary={}, Secondary={}, Timeout={}ms",
        primary, secondary, timeout_ms
    );

    // 上位DNSリゾルバー作成
    let upstream_resolver = UpstreamResolver::new(upstream_config);

    // DNSハンドラー作成（上位転送機能付き）
    let dns_handler = DnsHandler::new(cache.clone(), log_worker)
        .with_upstream(upstream_resolver);
    info!("DNSハンドラー初期化完了");

    // DNSサーバー起動 (UDP)
    let dns_addr = SocketAddr::from(([127, 0, 0, 1], 53));
    let udp_socket = UdpSocket::bind(dns_addr)
        .await
        .context("DNSサーバー(UDP)のバインドに失敗")?;
    info!("DNSサーバー(UDP)起動: {}", dns_addr);

    // DNSサーバー起動 (TCP)
    let tcp_listener = TokioTcpListener::bind(dns_addr)
        .await
        .context("DNSサーバー(TCP)のバインドに失敗")?;
    info!("DNSサーバー(TCP)起動: {}", dns_addr);

    // hickory-server の ServerFuture 作成
    let mut dns_server = ServerFuture::new(dns_handler);
    dns_server.register_socket(udp_socket);
    dns_server.register_listener(tcp_listener, std::time::Duration::from_secs(5));

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

    // DNSサーバーとWebサーバーを並行実行
    tokio::select! {
        result = dns_server.block_until_done() => {
            result.context("DNSサーバーの実行に失敗")?;
        }
        result = axum::serve(listener, app).with_graceful_shutdown(shutdown_signal()) => {
            result.context("Webサーバーの実行に失敗")?;
        }
        _ = shutdown_signal() => {
            info!("シャットダウンシグナルを受信しました");
        }
    }

    info!("LocalDNS Pro を終了します");
    Ok(())
}

/// シャットダウンシグナルを待機
async fn shutdown_signal() {
    let ctrl_c = async {
        signal::ctrl_c()
            .await
            .expect("Ctrl+Cハンドラーの登録に失敗");
    };

    #[cfg(unix)]
    let terminate = async {
        signal::unix::signal(signal::unix::SignalKind::terminate())
            .expect("SIGTERMハンドラーの登録に失敗")
            .recv()
            .await;
    };

    #[cfg(not(unix))]
    let terminate = std::future::pending::<()>();

    tokio::select! {
        _ = ctrl_c => {
            warn!("Ctrl+C受信、グレースフルシャットダウンを開始します");
        },
        _ = terminate => {
            warn!("SIGTERM受信、グレースフルシャットダウンを開始します");
        },
    }
}
