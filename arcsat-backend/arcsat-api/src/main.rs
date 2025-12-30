//! Arcsat API Server
//!
//! Servidor principal do ERP/CRM com Market Intelligence integrado

use axum::{
    routing::get,
    Router,
};
use std::sync::Arc;
use tower_http::cors::CorsLayer;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Carregar .env
    dotenv::dotenv().ok();

    // Setup logging
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "arcsat=debug,tower_http=debug".into()),
        )
        .with(tracing_subscriber::fmt::layer().json())
        .init();

    tracing::info!("🚀 Starting Arcsat API Server");

    // Config
    let config = arcsat_core::config::Config::default();
    let redis_url = std::env::var("REDIS_URL")
        .unwrap_or_else(|_| config.redis.url.clone());

    // Market Intelligence Engine
    let proxy_config = if config.market_intelligence.proxy_enabled {
        config.market_intelligence.proxy_url.as_ref().map(|url| {
            arcsat_market_intelligence::proxy::ProxyConfig::new(url.clone())
        })
    } else {
        None
    };

    let mi_engine = Arc::new(
        arcsat_market_intelligence::MarketIntelligenceEngine::new(&redis_url, proxy_config)
            .await?
    );

    tracing::info!("✅ Market Intelligence Engine initialized");

    // Router principal
    let app = Router::new()
        .route("/", get(root))
        .route("/health", get(health))
        .route("/api/v1/health", get(health))  // Health check alternativo
        .merge(arcsat_market_intelligence::api::router(mi_engine.clone()))
        .layer(CorsLayer::permissive());

    // Usar PORT do environment (Railway) ou config
    let port = std::env::var("PORT")
        .ok()
        .and_then(|p| p.parse::<u16>().ok())
        .unwrap_or(config.server.port);
    let addr = format!("{}:{}", config.server.host, port);
    tracing::info!("🌐 Server listening on {}", addr);

    let listener = tokio::net::TcpListener::bind(&addr).await?;
    axum::serve(listener, app).await?;

    Ok(())
}

async fn root() -> &'static str {
    "Arcsat ERP/CRM API - Market Intelligence Enabled"
}

async fn health() -> axum::Json<serde_json::Value> {
    axum::Json(serde_json::json!({
        "status": "healthy",
        "service": "arcsat-api",
        "timestamp": chrono::Utc::now(),
    }))
}
