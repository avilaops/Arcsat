// routes/analytics.rs - Analytics Avançado e KPIs

use axum::{
    extract::{Path, Query, State},
    response::Json,
    routing::get,
    Router,
};
use chrono::{DateTime, Datelike, Duration, NaiveDate, Utc};
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use uuid::Uuid;

use crate::{
    auth::Claims,
    cache::{CacheKeys, CacheManager, CacheTTL},
    db::DbPool,
    error::{AppError, Result},
};

pub fn routes(pool: DbPool, cache: CacheManager) -> Router {
    Router::new()
        // KPIs Financeiros
        .route("/kpis/financial", get(financial_kpis))
        .route("/kpis/sales", get(sales_kpis))
        .route("/kpis/hr", get(hr_kpis))
        
        // Análises
        .route("/cohort-analysis", get(cohort_analysis))
        .route("/churn-prediction", get(churn_prediction))
        .route("/revenue-forecast", get(revenue_forecast))
        
        // Relatórios
        .route("/reports/dashboard", get(executive_dashboard))
        .route("/reports/custom/:id", get(custom_report))
        
        .with_state(Arc::new((pool, cache)))
}

// ============================================================================
// KPIs FINANCEIROS
// ============================================================================

#[derive(Debug, Serialize)]
pub struct FinancialKPIs {
    pub period: String,
    pub mrr: KPIMetric,
    pub arr: KPIMetric,
    pub cac: KPIMetric,
    pub ltv: KPIMetric,
    pub ltv_cac_ratio: f64,
    pub churn_rate: KPIMetric,
    pub burn_rate: BurnRateMetric,
    pub gross_margin: f64,
    pub rule_of_40: f64, // Growth rate + Profit margin
}

#[derive(Debug, Serialize)]
pub struct KPIMetric {
    pub value: f64,
    pub change: f64,        // % change vs previous period
    pub trend: String,      // up, down, stable
    pub target: Option<f64>, // Meta estabelecida
}

#[derive(Debug, Serialize)]
pub struct BurnRateMetric {
    pub value: f64,
    pub runway_months: f64,
    pub cash_balance: f64,
}

async fn financial_kpis(
    claims: Claims,
    State((pool, cache)): State<Arc<(DbPool, CacheManager)>>,
    Query(query): Query<serde_json::Value>,
) -> Result<Json<FinancialKPIs>> {
    let tenant_id = Uuid::parse_str(&claims.tenant_id)?;
    
    // Tentar buscar do cache
    let cache_key = format!("analytics:{}:financial_kpis", tenant_id);
    if let Some(cached) = cache.get::<FinancialKPIs>(&cache_key).await? {
        return Ok(Json(cached));
    }

    // Calcular KPIs (simplificado - em produção, queries reais)
    let mrr = calculate_mrr(&pool, tenant_id).await?;
    let arr = mrr.value * 12.0;
    let cac = calculate_cac(&pool, tenant_id).await?;
    let ltv = calculate_ltv(&pool, tenant_id).await?;
    let churn_rate = calculate_churn_rate(&pool, tenant_id).await?;
    let burn_rate = calculate_burn_rate(&pool, tenant_id).await?;

    let kpis = FinancialKPIs {
        period: Utc::now().format("%Y-%m").to_string(),
        mrr,
        arr: KPIMetric {
            value: arr,
            change: mrr.change,
            trend: mrr.trend.clone(),
            target: Some(arr * 1.3), // Meta: crescer 30%
        },
        cac,
        ltv,
        ltv_cac_ratio: if cac.value > 0.0 { ltv.value / cac.value } else { 0.0 },
        churn_rate,
        burn_rate,
        gross_margin: 65.0, // Simulado
        rule_of_40: 45.0,   // Simulado (crescimento 25% + margem 20%)
    };

    // Salvar no cache (10 minutos)
    cache.set(&cache_key, &kpis, CacheTTL::forecast()).await?;

    Ok(Json(kpis))
}

// ============================================================================
// KPIs DE VENDAS
// ============================================================================

#[derive(Debug, Serialize)]
pub struct SalesKPIs {
    pub period: String,
    pub total_deals: u32,
    pub won_deals: u32,
    pub win_rate: f64,
    pub average_deal_size: f64,
    pub sales_cycle_days: f64,
    pub pipeline_value: f64,
    pub pipeline_velocity: f64, // $ por dia
    pub conversion_rate_by_stage: ConversionRates,
}

#[derive(Debug, Serialize)]
pub struct ConversionRates {
    pub new_to_contacted: f64,
    pub contacted_to_qualified: f64,
    pub qualified_to_proposal: f64,
    pub proposal_to_won: f64,
}

async fn sales_kpis(
    claims: Claims,
    State((pool, cache)): State<Arc<(DbPool, CacheManager)>>,
) -> Result<Json<SalesKPIs>> {
    let tenant_id = Uuid::parse_str(&claims.tenant_id)?;

    // Query simplificada (em produção, otimizar)
    let deals = sqlx::query!(
        r#"
        SELECT 
            COUNT(*) as "total!",
            COUNT(*) FILTER (WHERE stage = 'won') as "won!",
            AVG(value) FILTER (WHERE stage = 'won') as avg_value,
            SUM(value) FILTER (WHERE stage NOT IN ('won', 'lost')) as "pipeline_value!"
        FROM crm_leads
        WHERE tenant_id = $1
          AND created_at >= CURRENT_DATE - INTERVAL '30 days'
        "#,
        tenant_id
    )
    .fetch_one(pool.as_ref())
    .await?;

    let total_deals = deals.total as u32;
    let won_deals = deals.won as u32;
    let win_rate = if total_deals > 0 {
        (won_deals as f64 / total_deals as f64) * 100.0
    } else {
        0.0
    };

    let kpis = SalesKPIs {
        period: "Last 30 days".to_string(),
        total_deals,
        won_deals,
        win_rate,
        average_deal_size: deals.avg_value.map(|d| d.to_f64().unwrap_or(0.0)).unwrap_or(0.0),
        sales_cycle_days: 45.0, // Simulado
        pipeline_value: deals.pipeline_value.to_f64().unwrap_or(0.0),
        pipeline_velocity: 15000.0, // Simulado
        conversion_rate_by_stage: ConversionRates {
            new_to_contacted: 85.0,
            contacted_to_qualified: 60.0,
            qualified_to_proposal: 40.0,
            proposal_to_won: 70.0,
        },
    };

    Ok(Json(kpis))
}

// ============================================================================
// KPIs DE RH
// ============================================================================

#[derive(Debug, Serialize)]
pub struct HRKPIs {
    pub period: String,
    pub total_employees: u32,
    pub active_employees: u32,
    pub headcount_growth: f64,
    pub turnover_rate: f64,
    pub voluntary_turnover: f64,
    pub involuntary_turnover: f64,
    pub average_tenure_months: f64,
    pub time_to_hire_days: f64,
    pub cost_per_hire: f64,
    pub employee_satisfaction: f64,
}

async fn hr_kpis(
    claims: Claims,
    State((pool, _cache)): State<Arc<(DbPool, CacheManager)>>,
) -> Result<Json<HRKPIs>> {
    let tenant_id = Uuid::parse_str(&claims.tenant_id)?;

    let employees = sqlx::query!(
        r#"
        SELECT 
            COUNT(*) as "total!",
            COUNT(*) FILTER (WHERE status = 'active') as "active!"
        FROM hr_employees
        WHERE tenant_id = $1
        "#,
        tenant_id
    )
    .fetch_one(pool.as_ref())
    .await?;

    let kpis = HRKPIs {
        period: Utc::now().format("%Y-%m").to_string(),
        total_employees: employees.total as u32,
        active_employees: employees.active as u32,
        headcount_growth: 5.2, // Simulado
        turnover_rate: 12.5,   // Simulado
        voluntary_turnover: 8.0,
        involuntary_turnover: 4.5,
        average_tenure_months: 36.0,
        time_to_hire_days: 25.0,
        cost_per_hire: 3500.0,
        employee_satisfaction: 4.2, // 0-5
    };

    Ok(Json(kpis))
}

// ============================================================================
// ANÁLISE DE COHORT
// ============================================================================

#[derive(Debug, Serialize)]
pub struct CohortAnalysis {
    pub cohorts: Vec<Cohort>,
    pub retention_by_month: Vec<Vec<f64>>,
}

#[derive(Debug, Serialize)]
pub struct Cohort {
    pub cohort_month: String,
    pub customer_count: u32,
    pub initial_mrr: f64,
    pub current_mrr: f64,
    pub retention_rate: f64,
}

async fn cohort_analysis(
    claims: Claims,
    State((pool, _cache)): State<Arc<(DbPool, CacheManager)>>,
) -> Result<Json<CohortAnalysis>> {
    let tenant_id = Uuid::parse_str(&claims.tenant_id)?;

    // TODO: Implementar análise real de cohort
    // Por enquanto, dados simulados

    let cohorts = vec![
        Cohort {
            cohort_month: "2024-01".to_string(),
            customer_count: 25,
            initial_mrr: 50000.0,
            current_mrr: 48000.0,
            retention_rate: 96.0,
        },
        Cohort {
            cohort_month: "2024-02".to_string(),
            customer_count: 30,
            initial_mrr: 60000.0,
            current_mrr: 59000.0,
            retention_rate: 98.3,
        },
    ];

    let analysis = CohortAnalysis {
        cohorts,
        retention_by_month: vec![
            vec![100.0, 96.0, 92.0, 88.0],
            vec![100.0, 98.3, 95.0, 0.0],
        ],
    };

    Ok(Json(analysis))
}

// ============================================================================
// PREVISÃO DE CHURN
// ============================================================================

#[derive(Debug, Serialize)]
pub struct ChurnPrediction {
    pub accounts_at_risk: Vec<AccountRisk>,
    pub predicted_churn_rate: f64,
    pub revenue_at_risk: f64,
}

#[derive(Debug, Serialize)]
pub struct AccountRisk {
    pub account_id: Uuid,
    pub account_name: String,
    pub churn_probability: f64,
    pub mrr: f64,
    pub risk_factors: Vec<String>,
    pub recommended_actions: Vec<String>,
}

async fn churn_prediction(
    claims: Claims,
    State((pool, _cache)): State<Arc<(DbPool, CacheManager)>>,
) -> Result<Json<ChurnPrediction>> {
    let tenant_id = Uuid::parse_str(&claims.tenant_id)?;

    // TODO: Implementar modelo de ML real para previsão de churn
    // Por enquanto, simulado baseado em health score baixo

    let at_risk_accounts = sqlx::query!(
        r#"
        SELECT id, name, health_score
        FROM crm_accounts
        WHERE tenant_id = $1
          AND health_score < 60
          AND status = 'active'
        ORDER BY health_score ASC
        LIMIT 10
        "#,
        tenant_id
    )
    .fetch_all(pool.as_ref())
    .await?;

    let accounts_at_risk: Vec<AccountRisk> = at_risk_accounts
        .into_iter()
        .map(|acc| AccountRisk {
            account_id: acc.id,
            account_name: acc.name,
            churn_probability: (100.0 - acc.health_score.unwrap_or(50) as f64) / 100.0,
            mrr: 5000.0, // Simulado
            risk_factors: vec![
                "Low product usage".to_string(),
                "Negative support interactions".to_string(),
            ],
            recommended_actions: vec![
                "Schedule QBR".to_string(),
                "Offer training session".to_string(),
            ],
        })
        .collect();

    let revenue_at_risk: f64 = accounts_at_risk.iter().map(|a| a.mrr).sum();

    let prediction = ChurnPrediction {
        accounts_at_risk,
        predicted_churn_rate: 8.5,
        revenue_at_risk,
    };

    Ok(Json(prediction))
}

// ============================================================================
// PREVISÃO DE RECEITA
// ============================================================================

#[derive(Debug, Serialize)]
pub struct RevenueForecast {
    pub forecasts: Vec<MonthlyForecast>,
    pub confidence_interval: ConfidenceInterval,
    pub assumptions: Vec<String>,
}

#[derive(Debug, Serialize)]
pub struct MonthlyForecast {
    pub month: String,
    pub predicted_revenue: f64,
    pub new_business: f64,
    pub expansion: f64,
    pub churn: f64,
}

#[derive(Debug, Serialize)]
pub struct ConfidenceInterval {
    pub p10: f64,  // Pessimista
    pub p50: f64,  // Mediana
    pub p90: f64,  // Otimista
}

async fn revenue_forecast(
    claims: Claims,
    State((pool, _cache)): State<Arc<(DbPool, CacheManager)>>,
) -> Result<Json<RevenueForecast>> {
    let tenant_id = Uuid::parse_str(&claims.tenant_id)?;

    // TODO: Implementar forecast com ML/time series
    // Por enquanto, crescimento linear simulado

    let mut forecasts = Vec::new();
    let base_revenue = 850000.0;
    let growth_rate = 0.05; // 5% ao mês

    for i in 1..=12 {
        let month = (Utc::now() + Duration::days(i * 30))
            .format("%Y-%m")
            .to_string();
        
        let predicted_revenue = base_revenue * (1.0 + growth_rate).powi(i);

        forecasts.push(MonthlyForecast {
            month,
            predicted_revenue,
            new_business: predicted_revenue * 0.15,
            expansion: predicted_revenue * 0.10,
            churn: predicted_revenue * 0.08,
        });
    }

    let forecast = RevenueForecast {
        forecasts,
        confidence_interval: ConfidenceInterval {
            p10: base_revenue * 0.85,
            p50: base_revenue,
            p90: base_revenue * 1.25,
        },
        assumptions: vec![
            "5% monthly growth rate".to_string(),
            "8% churn rate".to_string(),
            "15% from new business".to_string(),
        ],
    };

    Ok(Json(forecast))
}

// ============================================================================
// DASHBOARD EXECUTIVO
// ============================================================================

#[derive(Debug, Serialize)]
pub struct ExecutiveDashboard {
    pub summary: DashboardSummary,
    pub financial_kpis: serde_json::Value,
    pub sales_kpis: serde_json::Value,
    pub top_opportunities: Vec<serde_json::Value>,
    pub alerts: Vec<Alert>,
}

#[derive(Debug, Serialize)]
pub struct DashboardSummary {
    pub total_revenue_ytd: f64,
    pub total_customers: u32,
    pub total_employees: u32,
    pub cash_balance: f64,
}

#[derive(Debug, Serialize)]
pub struct Alert {
    pub severity: String, // high, medium, low
    pub title: String,
    pub message: String,
    pub action_url: Option<String>,
}

async fn executive_dashboard(
    claims: Claims,
    State((pool, cache)): State<Arc<(DbPool, CacheManager)>>,
) -> Result<Json<ExecutiveDashboard>> {
    let tenant_id = Uuid::parse_str(&claims.tenant_id)?;

    // Agregar dados de múltiplas fontes
    let dashboard = ExecutiveDashboard {
        summary: DashboardSummary {
            total_revenue_ytd: 8500000.0,
            total_customers: 125,
            total_employees: 85,
            cash_balance: 1250000.0,
        },
        financial_kpis: serde_json::json!({}), // TODO: Buscar real
        sales_kpis: serde_json::json!({}),     // TODO: Buscar real
        top_opportunities: vec![],             // TODO: Buscar real
        alerts: vec![
            Alert {
                severity: "high".to_string(),
                title: "Cash Runway Alert".to_string(),
                message: "Current runway: 8 months. Consider fundraising.".to_string(),
                action_url: Some("/finance/cashflow".to_string()),
            },
        ],
    };

    Ok(Json(dashboard))
}

// ============================================================================
// RELATÓRIOS CUSTOMIZADOS
// ============================================================================

async fn custom_report(
    claims: Claims,
    State((pool, _cache)): State<Arc<(DbPool, CacheManager)>>,
    Path(report_id): Path<Uuid>,
) -> Result<Json<serde_json::Value>> {
    let tenant_id = Uuid::parse_str(&claims.tenant_id)?;

    // TODO: Buscar configuração do relatório customizado
    // Executar queries dinâmicas

    Ok(Json(serde_json::json!({
        "report_id": report_id,
        "data": []
    })))
}

// ============================================================================
// CÁLCULOS DE KPIs
// ============================================================================

async fn calculate_mrr(pool: &DbPool, tenant_id: Uuid) -> Result<KPIMetric> {
    // TODO: Calcular MRR real
    Ok(KPIMetric {
        value: 850000.0,
        change: 5.2,
        trend: "up".to_string(),
        target: Some(900000.0),
    })
}

async fn calculate_cac(pool: &DbPool, tenant_id: Uuid) -> Result<KPIMetric> {
    // TODO: Calcular CAC real (marketing + sales cost / novos clientes)
    Ok(KPIMetric {
        value: 12500.0,
        change: -8.3,
        trend: "down".to_string(), // Baixar CAC é bom
        target: Some(10000.0),
    })
}

async fn calculate_ltv(pool: &DbPool, tenant_id: Uuid) -> Result<KPIMetric> {
    // TODO: Calcular LTV real (ARPU * customer lifetime)
    Ok(KPIMetric {
        value: 95000.0,
        change: 12.1,
        trend: "up".to_string(),
        target: Some(120000.0),
    })
}

async fn calculate_churn_rate(pool: &DbPool, tenant_id: Uuid) -> Result<KPIMetric> {
    // TODO: Calcular churn rate real
    Ok(KPIMetric {
        value: 2.3,
        change: -0.5,
        trend: "down".to_string(), // Baixar churn é bom
        target: Some(2.0),
    })
}

async fn calculate_burn_rate(pool: &DbPool, tenant_id: Uuid) -> Result<BurnRateMetric> {
    // TODO: Calcular burn rate real
    Ok(BurnRateMetric {
        value: 180000.0,
        runway_months: 18.5,
        cash_balance: 3330000.0,
    })
}
