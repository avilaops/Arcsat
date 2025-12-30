use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::Json,
    routing::{get, post},
    Router,
};
use std::sync::Arc;
use crate::models::*;
use crate::insights::CrmIntegrationService;
use arcsat_core::ApiResponse;
use uuid::Uuid;

pub fn router() -> Router {
    Router::new()
        .route("/api/v1/crm/products/:product_id/insights", get(get_product_insights))
        .route("/api/v1/crm/products/:product_id/suggested-price", get(get_suggested_price))
}

/// GET /api/v1/crm/products/:product_id/insights
async fn get_product_insights(
    Path(product_id): Path<Uuid>,
) -> Result<Json<ApiResponse<Vec<MarketInsight>>>, StatusCode> {
    // TODO: Buscar produto do banco
    // TODO: Buscar dados de mercado do Market Intelligence
    // TODO: Gerar insights

    // Mock para demonstração
    let insights = vec![
        MarketInsight {
            id: Uuid::new_v4(),
            product_id,
            insight_type: InsightType::PricingOpportunity,
            title: "Oportunidade de ajuste de preço".to_string(),
            description: "Seu preço está 15% acima da média do mercado".to_string(),
            suggested_action: "Reduzir preço para R$ 299.90".to_string(),
            priority: InsightPriority::High,
            data: serde_json::json!({
                "current_price": 349.90,
                "market_avg": 299.90,
                "diff_percent": 15.0
            }),
            created_at: chrono::Utc::now(),
        }
    ];

    Ok(Json(ApiResponse::success(insights)))
}

/// GET /api/v1/crm/products/:product_id/suggested-price
async fn get_suggested_price(
    Path(product_id): Path<Uuid>,
) -> Result<Json<ApiResponse<serde_json::Value>>, StatusCode> {
    // TODO: Implementar cálculo real

    let response = serde_json::json!({
        "product_id": product_id,
        "current_price": 349.90,
        "suggested_price": 299.90,
        "market_avg": 310.00,
        "reasoning": "Preço sugerido está 5% abaixo da média para aumentar competitividade",
        "expected_margin": 28.5
    });

    Ok(Json(ApiResponse::success(response)))
}
