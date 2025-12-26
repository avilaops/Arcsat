use anyhow::Context;
use axum::{
    extract::State,
    middleware,
    response::Json,
    routing::get,
    Router,
};
use std::{net::SocketAddr, sync::Arc, time::Duration};
use tower::ServiceBuilder;
use tower_http::{
    compression::CompressionLayer,
    cors::{Any, CorsLayer},
    trace::TraceLayer,
};

mod auth;
mod cache;
mod db;
mod error;
mod models;
// Módulos temporariamente desabilitados
// mod routes;
// mod webhooks;

// Módulo opcional: se tiver MongoDB
// mod mongodb;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Carregar variáveis de ambiente
    dotenv::dotenv().ok();

    // Configurar logging
    tracing_subscriber::fmt()
        .with_target(false)
        .with_thread_ids(true)
        .with_level(true)
        .compact()
        .init();

    tracing::info!("🚀 Iniciando ERP/CRM Faria Lima Backend...");

    // Conectar ao banco PostgreSQL
    let database_url = std::env::var("DATABASE_URL")
        .context("DATABASE_URL não definida. Configure no .env")?;

    let pool = db::create_pool(&database_url)
        .await
        .context("Erro ao conectar no SQLite")?;

    tracing::info!("✅ Conectado ao SQLite");

    // Executar migrations sempre
    db::run_migrations(&pool)
        .await
        .context("Erro ao executar migrations")?;

    // Health check inicial
    db::health_check(&pool).await.context("Database health check failed")?;

    // Auth config
    let auth_config = Arc::new(auth::AuthConfig::new());

    // Redis desabilitado temporariamente
    // TODO: Reabilitar quando tiver Redis instalado
    tracing::warn!("⚠️  Cache Redis desabilitado");

    // CORS configuration
    // CORS configuration simplificada (sem credentials)
    let cors = CorsLayer::new()
        .allow_origin(Any)
        .allow_methods(Any)
        .allow_headers(Any);

    // API Routes
    let api_v1 = Router::new()
        // Rota de teste
        .route("/", get(|| async { "API v1 - ERP/CRM Faria Lima" }))

        // CRM (temporariamente desabilitado para compilação)
        // .nest("/crm", routes::crm::routes(pool.clone(), cache.clone()))

        // Finance (temporariamente desabilitado para compilação)
        // .nest("/finance", routes::finance::routes(pool.clone()))

        // HR (temporariamente desabilitado para compilação)
        // .nest("/hr", routes::hr::routes(pool.clone()))

        // Webhooks (temporariamente desabilitado para compilação)
        // .nest("/webhooks", webhooks::routes(pool.clone()))

        // TODO: Adicionar mais rotas
        // .nest("/analytics", routes::analytics::routes(pool.clone()))

        // Middleware de autenticação (desabilitado temporariamente)
        // .layer(middleware::from_fn(auth_middleware))
        ;

    // Auth routes (sem autenticação)
    let auth_routes = auth::auth_routes()
        .with_state(auth_config.clone());

    // Health & Metrics
    let health_routes = Router::new()
        .route("/health", get(health_check))
        .route("/health/db", get(health_check_db))
        .route("/metrics", get(metrics))
        .with_state(pool.clone());

    // App principal
    let app = Router::new()
        .nest("/api/v1", api_v1)
        .nest("/api/v1/auth", auth_routes)
        .merge(health_routes)
        .layer(
            ServiceBuilder::new()
                .layer(TraceLayer::new_for_http())
                .layer(CompressionLayer::new())
                .layer(cors),
        );

    // Bind e servir
    let host = std::env::var("HOST").unwrap_or_else(|_| "0.0.0.0".to_string());
    let port = std::env::var("PORT")
        .ok()
        .and_then(|p| p.parse::<u16>().ok())
        .unwrap_or(3000);

    let addr: SocketAddr = format!("{}:{}", host, port)
        .parse()
        .context("HOST ou PORT inválidos")?;

    tracing::info!("🌐 Servidor rodando em http://{}", addr);
    tracing::info!("📊 Health check: http://{}/health", addr);
    tracing::info!("📖 API: http://{}/api/v1", addr);

    let listener = tokio::net::TcpListener::bind(addr).await?;
    axum::serve(listener, app).await?;

    Ok(())
}

// ============================================================================
// MIDDLEWARE
// ============================================================================

/// Middleware de autenticação (extrai e valida JWT)
async fn auth_middleware(
    mut request: axum::extract::Request,
    next: axum::middleware::Next,
) -> Result<axum::response::Response, error::AppError> {
    use axum::http::header;

    // Extrair token do header Authorization
    let auth_header = request
        .headers()
        .get(header::AUTHORIZATION)
        .and_then(|h| h.to_str().ok());

    if let Some(auth_value) = auth_header {
        if let Some(token) = auth_value.strip_prefix("Bearer ") {
            // Validar token
            let config = auth::AuthConfig::new();
            match config.validate_token(token) {
                Ok(claims) => {
                    // Adicionar claims nas extensions
                    request.extensions_mut().insert(claims);
                }
                Err(e) => {
                    tracing::warn!("Invalid token: {}", e);
                    return Err(e);
                }
            }
        }
    }

    Ok(next.run(request).await)
}

// ============================================================================
// HEALTH CHECKS
// ============================================================================

async fn health_check() -> &'static str {
    "OK"
}

async fn health_check_db(
    State(pool): State<db::DbPool>,
) -> Result<Json<serde_json::Value>, error::AppError> {
    db::health_check(&pool).await?;

    let stats = db::pool_stats(&pool);

    Ok(Json(serde_json::json!({
        "status": "healthy",
        "database": {
            "connected": true,
            "pool": stats
        }
    })))
}

async fn metrics(
    State(pool): State<db::DbPool>,
) -> Json<serde_json::Value> {
    let stats = db::pool_stats(&pool);

    Json(serde_json::json!({
        "database": {
            "connections": {
                "total": stats.size,
                "idle": stats.idle,
                "active": stats.active
            }
        },
        "uptime_seconds": 0, // TODO: Implementar
        "requests_total": 0, // TODO: Implementar com metrics crate
    }))
}
