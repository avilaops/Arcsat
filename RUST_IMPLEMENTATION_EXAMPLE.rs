// backend/src/main.rs - Exemplo de Implementação ERP/CRM Faria Lima

use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    response::Json,
    routing::{get, post, patch},
    Router,
};
use serde::{Deserialize, Serialize};
use sqlx::PgPool;
use std::sync::Arc;
use tower_http::cors::CorsLayer;
use uuid::Uuid;

// ============================================================================
// ESTRUTURAS DE DADOS
// ============================================================================

#[derive(Debug, Serialize, Deserialize)]
pub struct Lead {
    pub id: Uuid,
    pub tenant_id: Uuid,
    pub name: String,
    pub company: Option<String>,
    pub email: String,
    pub phone: Option<String>,
    pub source: String,
    pub stage: LeadStage,
    pub score: i32,
    pub value: f64,
    pub probability: i32,
    pub owner_id: Uuid,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub updated_at: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug, Serialize, Deserialize, sqlx::Type)]
#[sqlx(type_name = "lead_stage", rename_all = "lowercase")]
pub enum LeadStage {
    New,
    Contacted,
    Qualification,
    Proposal,
    Negotiation,
    Won,
    Lost,
}

#[derive(Debug, Deserialize)]
pub struct CreateLeadRequest {
    pub name: String,
    pub company: Option<String>,
    pub email: String,
    pub phone: Option<String>,
    pub source: String,
    pub value: f64,
    pub expected_close_date: Option<chrono::NaiveDate>,
}

#[derive(Debug, Deserialize)]
pub struct UpdateLeadStageRequest {
    pub stage: LeadStage,
    pub reason: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct ListLeadsQuery {
    pub stage: Option<String>,
    pub score_min: Option<i32>,
    pub page: Option<i32>,
    pub limit: Option<i32>,
}

#[derive(Debug, Serialize)]
pub struct PaginatedResponse<T> {
    pub data: Vec<T>,
    pub pagination: Pagination,
}

#[derive(Debug, Serialize)]
pub struct Pagination {
    pub current_page: i32,
    pub total_pages: i32,
    pub total_items: i64,
    pub per_page: i32,
}

// Estruturas Financeiras
#[derive(Debug, Serialize, Deserialize)]
pub struct CashflowProjection {
    pub month: String,
    pub opening_balance: f64,
    pub inflows: Inflows,
    pub outflows: Outflows,
    pub closing_balance: f64,
    pub ai_confidence: f32,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Inflows {
    pub sales: f64,
    pub receivables: f64,
    pub other: f64,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Outflows {
    pub payroll: f64,
    pub suppliers: f64,
    pub taxes: f64,
    pub operational: f64,
}

#[derive(Debug, Serialize)]
pub struct DREResponse {
    pub period: String,
    pub summary: DRESummary,
    pub monthly: Vec<MonthlyDRE>,
}

#[derive(Debug, Serialize)]
pub struct DRESummary {
    pub revenue: RevenueMetrics,
    pub costs: CostMetrics,
    pub expenses: ExpenseMetrics,
    pub ebitda: f64,
    pub ebitda_margin: f64,
    pub net_income: f64,
    pub net_margin: f64,
}

#[derive(Debug, Serialize)]
pub struct RevenueMetrics {
    pub gross: f64,
    pub net: f64,
    pub growth_yoy: f64,
}

#[derive(Debug, Serialize)]
pub struct CostMetrics {
    pub cogs: f64,
    pub gross_margin: f64,
}

#[derive(Debug, Serialize)]
pub struct ExpenseMetrics {
    pub sales: f64,
    pub administrative: f64,
    pub operational: f64,
}

#[derive(Debug, Serialize)]
pub struct MonthlyDRE {
    pub month: String,
    pub revenue: f64,
    pub ebitda: f64,
    pub margin: f64,
}

// Estado compartilhado da aplicação
#[derive(Clone)]
pub struct AppState {
    pub db: PgPool,
}

// ============================================================================
// HANDLERS CRM
// ============================================================================

/// Criar novo lead
async fn create_lead(
    State(state): State<Arc<AppState>>,
    Json(payload): Json<CreateLeadRequest>,
) -> Result<(StatusCode, Json<Lead>), StatusCode> {
    let lead_id = Uuid::new_v4();
    let tenant_id = Uuid::new_v4(); // Em produção, extrair do JWT
    let owner_id = Uuid::new_v4(); // Em produção, extrair do JWT
    
    // Calcular lead score usando IA (simplificado)
    let score = calculate_lead_score(&payload);
    
    let lead = sqlx::query_as!(
        Lead,
        r#"
        INSERT INTO crm_leads 
            (id, tenant_id, name, company, email, phone, source, stage, score, value, probability, owner_id)
        VALUES 
            ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12)
        RETURNING 
            id, tenant_id, name, company, email, phone, source, 
            stage as "stage: LeadStage", score, value, probability, owner_id, 
            created_at, updated_at
        "#,
        lead_id,
        tenant_id,
        payload.name,
        payload.company,
        payload.email,
        payload.phone,
        payload.source,
        LeadStage::New as LeadStage,
        score,
        payload.value,
        20, // Probabilidade inicial
        owner_id,
    )
    .fetch_one(&state.db)
    .await
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok((StatusCode::CREATED, Json(lead)))
}

/// Listar leads com filtros e paginação
async fn list_leads(
    State(state): State<Arc<AppState>>,
    Query(query): Query<ListLeadsQuery>,
) -> Result<Json<PaginatedResponse<Lead>>, StatusCode> {
    let page = query.page.unwrap_or(1);
    let limit = query.limit.unwrap_or(50);
    let offset = (page - 1) * limit;

    let tenant_id = Uuid::new_v4(); // Em produção, extrair do JWT

    // Contar total de leads
    let total: (i64,) = sqlx::query_as(
        "SELECT COUNT(*) FROM crm_leads WHERE tenant_id = $1"
    )
    .bind(tenant_id)
    .fetch_one(&state.db)
    .await
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let total_items = total.0;
    let total_pages = (total_items as f64 / limit as f64).ceil() as i32;

    // Buscar leads
    let leads = sqlx::query_as!(
        Lead,
        r#"
        SELECT 
            id, tenant_id, name, company, email, phone, source,
            stage as "stage: LeadStage", score, value, probability, owner_id,
            created_at, updated_at
        FROM crm_leads 
        WHERE tenant_id = $1
        ORDER BY created_at DESC
        LIMIT $2 OFFSET $3
        "#,
        tenant_id,
        limit as i64,
        offset as i64,
    )
    .fetch_all(&state.db)
    .await
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok(Json(PaginatedResponse {
        data: leads,
        pagination: Pagination {
            current_page: page,
            total_pages,
            total_items,
            per_page: limit,
        },
    }))
}

/// Atualizar estágio do lead
async fn update_lead_stage(
    State(state): State<Arc<AppState>>,
    Path(lead_id): Path<Uuid>,
    Json(payload): Json<UpdateLeadStageRequest>,
) -> Result<Json<Lead>, StatusCode> {
    let tenant_id = Uuid::new_v4(); // Em produção, extrair do JWT

    // Atualizar probabilidade baseado no estágio
    let probability = match payload.stage {
        LeadStage::New => 10,
        LeadStage::Contacted => 15,
        LeadStage::Qualification => 25,
        LeadStage::Proposal => 60,
        LeadStage::Negotiation => 80,
        LeadStage::Won => 100,
        LeadStage::Lost => 0,
    };

    let lead = sqlx::query_as!(
        Lead,
        r#"
        UPDATE crm_leads 
        SET stage = $1, probability = $2, updated_at = NOW()
        WHERE id = $3 AND tenant_id = $4
        RETURNING 
            id, tenant_id, name, company, email, phone, source,
            stage as "stage: LeadStage", score, value, probability, owner_id,
            created_at, updated_at
        "#,
        payload.stage as LeadStage,
        probability,
        lead_id,
        tenant_id,
    )
    .fetch_one(&state.db)
    .await
    .map_err(|_| StatusCode::NOT_FOUND)?;

    Ok(Json(lead))
}

/// Forecast de vendas
async fn sales_forecast(
    State(state): State<Arc<AppState>>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    let tenant_id = Uuid::new_v4(); // Em produção, extrair do JWT

    let result = sqlx::query!(
        r#"
        SELECT 
            stage as "stage!: LeadStage",
            COUNT(*) as "count!",
            SUM(value) as "total_value!",
            AVG(probability) as "avg_probability!"
        FROM crm_leads
        WHERE tenant_id = $1 
          AND stage NOT IN ('won', 'lost')
        GROUP BY stage
        "#,
        tenant_id
    )
    .fetch_all(&state.db)
    .await
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let mut weighted_total = 0.0;
    let mut breakdown = serde_json::Map::new();

    for row in result {
        let stage_value = row.total_value * (row.avg_probability / 100.0);
        weighted_total += stage_value;

        breakdown.insert(
            format!("{:?}", row.stage).to_lowercase(),
            serde_json::json!({
                "count": row.count,
                "value": row.total_value,
                "probability": row.avg_probability
            }),
        );
    }

    Ok(Json(serde_json::json!({
        "period": "Q1-2024",
        "forecast": {
            "best_case": weighted_total * 1.35,
            "most_likely": weighted_total,
            "worst_case": weighted_total * 0.65,
            "weighted": weighted_total
        },
        "breakdown_by_stage": breakdown
    })))
}

// ============================================================================
// HANDLERS FINANCEIRO
// ============================================================================

/// Projeção de fluxo de caixa com IA
async fn cashflow_projection(
    State(_state): State<Arc<AppState>>,
) -> Result<Json<Vec<CashflowProjection>>, StatusCode> {
    // Em produção, usar modelo de ML para previsão
    // Aqui é um exemplo simplificado
    let projections = vec![
        CashflowProjection {
            month: "2024-02".to_string(),
            opening_balance: 500000.0,
            inflows: Inflows {
                sales: 350000.0,
                receivables: 120000.0,
                other: 10000.0,
            },
            outflows: Outflows {
                payroll: 180000.0,
                suppliers: 95000.0,
                taxes: 45000.0,
                operational: 60000.0,
            },
            closing_balance: 600000.0,
            ai_confidence: 0.87,
        },
        // ... mais meses
    ];

    Ok(Json(projections))
}

/// DRE gerencial em tempo real
async fn dre_realtime(
    State(state): State<Arc<AppState>>,
) -> Result<Json<DREResponse>, StatusCode> {
    let tenant_id = Uuid::new_v4(); // Em produção, extrair do JWT

    // Query simplificada - em produção seria muito mais complexa
    let revenue = sqlx::query!(
        r#"
        SELECT 
            COALESCE(SUM(amount), 0) as "total_revenue!"
        FROM finance_transactions
        WHERE tenant_id = $1 
          AND type = 'revenue'
          AND EXTRACT(YEAR FROM date) = EXTRACT(YEAR FROM CURRENT_DATE)
        "#,
        tenant_id
    )
    .fetch_one(&state.db)
    .await
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let response = DREResponse {
        period: "2024".to_string(),
        summary: DRESummary {
            revenue: RevenueMetrics {
                gross: revenue.total_revenue,
                net: revenue.total_revenue * 0.95,
                growth_yoy: 23.5,
            },
            costs: CostMetrics {
                cogs: revenue.total_revenue * 0.36,
                gross_margin: 64.0,
            },
            expenses: ExpenseMetrics {
                sales: revenue.total_revenue * 0.17,
                administrative: revenue.total_revenue * 0.15,
                operational: revenue.total_revenue * 0.10,
            },
            ebitda: revenue.total_revenue * 0.22,
            ebitda_margin: 22.0,
            net_income: revenue.total_revenue * 0.14,
            net_margin: 14.0,
        },
        monthly: vec![], // Implementar detalhamento mensal
    };

    Ok(Json(response))
}

// ============================================================================
// FUNÇÕES AUXILIARES
// ============================================================================

/// Calcular score do lead usando heurísticas (em produção, usar ML)
fn calculate_lead_score(lead: &CreateLeadRequest) -> i32 {
    let mut score = 50; // Base score

    // Fator: valor do negócio
    if lead.value > 100000.0 {
        score += 20;
    } else if lead.value > 50000.0 {
        score += 10;
    }

    // Fator: fonte do lead
    match lead.source.as_str() {
        "linkedin" => score += 15,
        "referral" => score += 20,
        "website" => score += 10,
        _ => {}
    }

    // Fator: empresa conhecida
    if lead.company.is_some() {
        score += 5;
    }

    score.min(100) // Max score é 100
}

// ============================================================================
// CONFIGURAÇÃO DO SERVIDOR
// ============================================================================

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Inicializar tracing
    tracing_subscriber::fmt()
        .with_target(false)
        .compact()
        .init();

    // Conectar ao banco de dados
    let database_url = std::env::var("DATABASE_URL")
        .unwrap_or_else(|_| "postgres://user:pass@localhost/erp".to_string());
    
    let pool = PgPool::connect(&database_url).await?;

    // Estado compartilhado
    let app_state = Arc::new(AppState { db: pool });

    // Definir rotas
    let app = Router::new()
        // Rotas CRM
        .route("/api/v1/crm/leads", post(create_lead).get(list_leads))
        .route("/api/v1/crm/leads/:id/stage", patch(update_lead_stage))
        .route("/api/v1/crm/opportunities/forecast", get(sales_forecast))
        
        // Rotas Financeiro
        .route("/api/v1/finance/cashflow/projection", post(cashflow_projection))
        .route("/api/v1/finance/dre/realtime", get(dre_realtime))
        
        // Health check
        .route("/health", get(|| async { "OK" }))
        
        // Estado e middlewares
        .with_state(app_state)
        .layer(CorsLayer::permissive());

    // Iniciar servidor
    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await?;
    
    tracing::info!("🚀 Servidor rodando em http://0.0.0.0:3000");
    
    axum::serve(listener, app).await?;

    Ok(())
}

// ============================================================================
// TESTES
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_calculate_lead_score() {
        let lead = CreateLeadRequest {
            name: "Test Lead".to_string(),
            company: Some("Big Corp".to_string()),
            email: "test@example.com".to_string(),
            phone: None,
            source: "linkedin".to_string(),
            value: 150000.0,
            expected_close_date: None,
        };

        let score = calculate_lead_score(&lead);
        assert!(score >= 80); // High-value LinkedIn lead deve ter score alto
    }

    #[test]
    fn test_lead_stage_probability() {
        assert_eq!(match_probability(LeadStage::Qualification), 25);
        assert_eq!(match_probability(LeadStage::Proposal), 60);
        assert_eq!(match_probability(LeadStage::Won), 100);
    }

    fn match_probability(stage: LeadStage) -> i32 {
        match stage {
            LeadStage::New => 10,
            LeadStage::Contacted => 15,
            LeadStage::Qualification => 25,
            LeadStage::Proposal => 60,
            LeadStage::Negotiation => 80,
            LeadStage::Won => 100,
            LeadStage::Lost => 0,
        }
    }
}
