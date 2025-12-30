use axum::{
    extract::{Path, State, Query},
    http::StatusCode,
    response::Json,
    routing::{get, post},
    Router,
};
use std::sync::Arc;
use serde::Deserialize;
use crate::models::*;
use crate::MarketIntelligenceEngine;
use arcsat_core::{ApiResponse, Result};

pub fn router(engine: Arc<MarketIntelligenceEngine>) -> Router {
    Router::new()
        .route("/api/v1/market-intelligence/jobs", post(create_job))
        .route("/api/v1/market-intelligence/jobs/:job_id", get(get_job))
        .route("/api/v1/market-intelligence/jobs/:job_id/status", get(get_job_status))
        .route("/api/v1/market-intelligence/trends", get(get_trends))
        .route("/api/v1/market-intelligence/health", get(health_check))
        .with_state(engine)
}

/// POST /api/v1/market-intelligence/jobs
async fn create_job(
    State(engine): State<Arc<MarketIntelligenceEngine>>,
    Json(request): Json<CreateJobRequest>,
) -> Result<Json<ApiResponse<CreateJobResponse>>, StatusCode> {
    // TODO: Extrair tenant_id do JWT
    let tenant_id = uuid::Uuid::new_v4();

    let mut job = ScrapingJob::new(
        tenant_id,
        request.marketplace,
        request.search_query,
        request.max_pages,
    );

    if let Some(category) = request.category {
        job.category = Some(category);
    }

    if let Some(priority) = request.priority {
        job.priority = priority.clamp(1, 10);
    }

    let job_id = engine.submit_job(job.clone())
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let response = CreateJobResponse {
        job_id: job.id,
        status: job.status,
        message: "Job criado e enfileirado para processamento".to_string(),
    };

    Ok(Json(ApiResponse::success(response)))
}

/// GET /api/v1/market-intelligence/jobs/:job_id/status
async fn get_job_status(
    State(engine): State<Arc<MarketIntelligenceEngine>>,
    Path(job_id): Path<String>,
) -> Result<Json<ApiResponse<JobStatus>>, StatusCode> {
    let status = engine.get_job_status(&job_id)
        .await
        .map_err(|_| StatusCode::NOT_FOUND)?;

    Ok(Json(ApiResponse::success(status)))
}

/// GET /api/v1/market-intelligence/jobs/:job_id
async fn get_job(
    State(engine): State<Arc<MarketIntelligenceEngine>>,
    Path(job_id): Path<String>,
) -> Result<Json<ApiResponse<String>>, StatusCode> {
    // TODO: Implementar busca completa do job
    Ok(Json(ApiResponse::success(format!("Job {}", job_id))))
}

#[derive(Debug, Deserialize)]
struct TrendsQuery {
    marketplace: Option<Marketplace>,
    category: Option<String>,
}

/// GET /api/v1/market-intelligence/trends
async fn get_trends(
    State(engine): State<Arc<MarketIntelligenceEngine>>,
    Query(query): Query<TrendsQuery>,
) -> Result<Json<ApiResponse<String>>, StatusCode> {
    // TODO: Implementar análise de tendências
    Ok(Json(ApiResponse::success("Trends analysis".to_string())))
}

/// GET /api/v1/market-intelligence/health
async fn health_check() -> Json<ApiResponse<String>> {
    Json(ApiResponse::success("Market Intelligence module is healthy".to_string()))
}
