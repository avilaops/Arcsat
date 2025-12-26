use crate::{
    auth::Claims,
    cache::{CacheInvalidation, CacheKeys, CacheManager, CacheTTL},
    db::DbPool,
    error::{AppError, Result},
    middleware::rate_limit::rate_limit_middleware,
    models::{self, *},
};
use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    middleware::from_fn_with_state,
    response::Json,
    routing::{get, patch, post},
    Router,
};
use chrono::Utc;
use rust_decimal::Decimal;
use std::sync::Arc;
use uuid::Uuid;
use validator::Validate;

// Importações explícitas dos enums
use crate::models::{ActivityStatus, LeadSource, LeadStage};

#[derive(Clone)]
struct CrmState {
    pool: DbPool,
    cache: Arc<CacheManager>,
}

pub fn routes(pool: DbPool, cache: Arc<CacheManager>) -> Router {
    let state = Arc::new(CrmState {
        pool: pool.clone(),
        cache: cache.clone(),
    });

    Router::new()
        .route("/leads", post(create_lead).get(list_leads))
        .route(
            "/leads/:id",
            get(get_lead).patch(update_lead).delete(delete_lead),
        )
        .route("/leads/:id/stage", patch(update_lead_stage))
        .route("/leads/:id/history", get(get_lead_history))
        .route("/opportunities/forecast", get(sales_forecast))
        .route("/opportunities/pipeline", get(pipeline_view))
        .route("/accounts", post(create_account).get(list_accounts))
        .route("/accounts/:id", get(get_account))
        .route("/accounts/:id/health", get(account_health_score))
        .route("/activities", post(create_activity).get(list_activities))
        .route("/activities/:id", get(get_activity).patch(update_activity))
        .route("/contacts", post(create_contact).get(list_contacts))
        .route("/contacts/:id", get(get_contact).patch(update_contact))
        .with_state(state)
        .layer(from_fn_with_state(cache, rate_limit_middleware))
}

async fn create_lead(
    claims: Claims,
    State(state): State<Arc<CrmState>>,
    Json(payload): Json<CreateLeadRequest>,
) -> Result<(StatusCode, Json<Lead>)> {
    payload
        .validate()
        .map_err(|e| AppError::validation_error(e.to_string()))?;

    let tenant_id_str = claims.tenant_id.clone();
    let tenant_id = Uuid::parse_str(&tenant_id_str)?;
    let owner_id = Uuid::parse_str(&claims.sub)?;

    let lead_id = Uuid::new_v4();
    let score = calculate_lead_score(&payload);
    let probability = calculate_probability(&LeadStage::New);
    let value = Decimal::from_f64_retain(payload.value)
        .ok_or_else(|| AppError::validation_error("Valor inválido para lead".to_string()))?;

    let lead = sqlx::query_as!(
        Lead,
        r#"
        INSERT INTO crm_leads
            (id, tenant_id, name, company, email, phone, source, stage, score,
             value, probability, owner_id, expected_close_date, custom_fields)
        VALUES
            ($1, $2, $3, $4, $5, $6, $7::lead_source, $8::lead_stage, $9,
             $10, $11, $12, $13, $14)
        RETURNING
            id, tenant_id, name, company, email, phone,
            source as "source: LeadSource",
            stage as "stage: LeadStage",
            score, value, probability, owner_id,
            expected_close_date, actual_close_date, lost_reason,
            custom_fields as "custom_fields: serde_json::Value",
            created_at, updated_at
        "#,
        lead_id,
        tenant_id,
        payload.name,
        payload.company,
        payload.email,
        payload.phone,
        payload.source as LeadSource,
        LeadStage::New as LeadStage,
        score,
        value,
        probability,
        owner_id,
        payload.expected_close_date,
        payload.custom_fields,
    )
    .fetch_one(&state.pool)
    .await?;

    CacheInvalidation::on_lead_modified(&state.cache, &tenant_id_str, &lead_id.to_string()).await?;

    tracing::info!(tenant = %tenant_id, lead_id = %lead_id, "Lead created");

    Ok((StatusCode::CREATED, Json(lead)))
}

async fn list_leads(
    claims: Claims,
    State(state): State<Arc<CrmState>>,
    Query(query): Query<ListLeadsQuery>,
) -> Result<Json<PaginatedResponse<LeadResponse>>> {
    let tenant_id_str = claims.tenant_id.clone();
    let tenant_id = Uuid::parse_str(&tenant_id_str)?;

    let ListLeadsQuery {
        stage,
        score_min,
        owner_id,
        source,
        page,
        limit,
    } = query;

    let page = page.unwrap_or(1).max(1);
    let limit = limit.unwrap_or(50).clamp(1, 100);
    let offset = (page - 1) * limit;

    let cache_key = format!(
        "{}:limit={}:stage={}:score_min={}:owner={}:source={}",
        CacheKeys::leads_list(&tenant_id_str, page),
        limit,
        stage
            .as_ref()
            .map(|s| format!("{:?}", s))
            .unwrap_or_else(|| "any".to_string()),
        score_min
            .map(|score| score.to_string())
            .unwrap_or_else(|| "any".to_string()),
        owner_id
            .as_ref()
            .map(|owner| owner.to_string())
            .unwrap_or_else(|| "any".to_string()),
        source
            .as_ref()
            .map(|s| format!("{:?}", s))
            .unwrap_or_else(|| "any".to_string()),
    );

    let pool = state.pool.clone();

    let response = state
        .cache
        .get_or_set(&cache_key, CacheTTL::leads_list(), move || {
            let pool = pool.clone();
            let stage = stage.clone();
            let score_min = score_min;
            let owner_id = owner_id;
            let source = source.clone();
            async move {
                let total_items = sqlx::query_scalar!(
                    r#"
                    SELECT COUNT(*)
                    FROM crm_leads
                    WHERE tenant_id = $1
                      AND ($2::lead_stage IS NULL OR stage = $2::lead_stage)
                      AND ($3::integer IS NULL OR score >= $3)
                      AND ($4::uuid IS NULL OR owner_id = $4)
                      AND ($5::lead_source IS NULL OR source = $5::lead_source)
                    "#,
                    tenant_id,
                    stage as Option<LeadStage>,
                    score_min,
                    owner_id,
                    source as Option<LeadSource>,
                )
                .fetch_one(&pool)
                .await?;

                let leads = sqlx::query_as!(
                    Lead,
                    r#"
                    SELECT
                        id, tenant_id, name, company, email, phone,
                        source as "source: LeadSource",
                        stage as "stage: LeadStage",
                        score, value, probability, owner_id,
                        expected_close_date, actual_close_date, lost_reason,
                        custom_fields as "custom_fields: serde_json::Value",
                        created_at, updated_at
                    FROM crm_leads
                    WHERE tenant_id = $1
                      AND ($2::lead_stage IS NULL OR stage = $2::lead_stage)
                      AND ($3::integer IS NULL OR score >= $3)
                      AND ($4::uuid IS NULL OR owner_id = $4)
                      AND ($5::lead_source IS NULL OR source = $5::lead_source)
                    ORDER BY score DESC, created_at DESC
                    LIMIT $6 OFFSET $7
                    "#,
                    tenant_id,
                    stage as Option<LeadStage>,
                    score_min,
                    owner_id,
                    source as Option<LeadSource>,
                    limit as i64,
                    offset as i64,
                )
                .fetch_all(&pool)
                .await?;

                let mut data = Vec::with_capacity(leads.len());
                for lead in leads {
                    let owner = get_user_summary(&pool, lead.owner_id).await?;
                    data.push(LeadResponse {
                        id: lead.id,
                        name: lead.name,
                        company: lead.company,
                        email: lead.email,
                        phone: lead.phone,
                        source: lead.source,
                        stage: lead.stage,
                        score: lead.score,
                        value: lead.value.to_string(),
                        probability: lead.probability,
                        owner,
                        created_at: lead.created_at,
                        last_contact: None,
                    });
                }

                Ok(PaginatedResponse::new(data, page, limit, total_items))
            }
        })
        .await?;

    Ok(Json(response))
}

async fn get_lead(
    claims: Claims,
    State(state): State<Arc<CrmState>>,
    Path(id): Path<Uuid>,
) -> Result<Json<Lead>> {
    let tenant_id = Uuid::parse_str(&claims.tenant_id)?;

    let lead = sqlx::query_as!(
        Lead,
        r#"
        SELECT
            id, tenant_id, name, company, email, phone,
            source as "source: LeadSource",
            stage as "stage: LeadStage",
            score, value, probability, owner_id,
            expected_close_date, actual_close_date, lost_reason,
            custom_fields as "custom_fields: serde_json::Value",
            created_at, updated_at
        FROM crm_leads
        WHERE id = $1 AND tenant_id = $2
        "#,
        id,
        tenant_id
    )
    .fetch_optional(&state.pool)
    .await?
    .ok_or_else(|| AppError::not_found("Lead"))?;

    Ok(Json(lead))
}

async fn update_lead(
    claims: Claims,
    State(state): State<Arc<CrmState>>,
    Path(id): Path<Uuid>,
    Json(payload): Json<CreateLeadRequest>,
) -> Result<Json<Lead>> {
    payload
        .validate()
        .map_err(|e| AppError::validation_error(e.to_string()))?;

    let tenant_id_str = claims.tenant_id.clone();
    let tenant_id = Uuid::parse_str(&tenant_id_str)?;

    let value = Decimal::from_f64_retain(payload.value)
        .ok_or_else(|| AppError::validation_error("Valor inválido para lead".to_string()))?;

    let lead = sqlx::query_as!(
        Lead,
        r#"
        UPDATE crm_leads
        SET
            name = $1,
            company = $2,
            email = $3,
            phone = $4,
            source = $5::lead_source,
            value = $6,
            expected_close_date = $7,
            custom_fields = $8,
            updated_at = NOW()
        WHERE id = $9 AND tenant_id = $10
        RETURNING
            id, tenant_id, name, company, email, phone,
            source as "source: LeadSource",
            stage as "stage: LeadStage",
            score, value, probability, owner_id,
            expected_close_date, actual_close_date, lost_reason,
            custom_fields as "custom_fields: serde_json::Value",
            created_at, updated_at
        "#,
        payload.name,
        payload.company,
        payload.email,
        payload.phone,
        payload.source as LeadSource,
        value,
        payload.expected_close_date,
        payload.custom_fields,
        id,
        tenant_id,
    )
    .fetch_optional(&state.pool)
    .await?
    .ok_or_else(|| AppError::not_found("Lead"))?;

    CacheInvalidation::on_lead_modified(&state.cache, &tenant_id_str, &id.to_string()).await?;

    tracing::info!(tenant = %tenant_id, lead_id = %id, "Lead updated");

    Ok(Json(lead))
}

async fn delete_lead(
    claims: Claims,
    State(state): State<Arc<CrmState>>,
    Path(id): Path<Uuid>,
) -> Result<StatusCode> {
    let tenant_id_str = claims.tenant_id.clone();
    let tenant_id = Uuid::parse_str(&tenant_id_str)?;

    let result = sqlx::query!(
        "DELETE FROM crm_leads WHERE id = $1 AND tenant_id = $2",
        id,
        tenant_id
    )
    .execute(&state.pool)
    .await?;

    if result.rows_affected() == 0 {
        return Err(AppError::not_found("Lead"));
    }

    CacheInvalidation::on_lead_modified(&state.cache, &tenant_id_str, &id.to_string()).await?;

    tracing::info!(tenant = %tenant_id, lead_id = %id, "Lead deleted");

    Ok(StatusCode::NO_CONTENT)
}

async fn update_lead_stage(
    claims: Claims,
    State(state): State<Arc<CrmState>>,
    Path(lead_id): Path<Uuid>,
    Json(payload): Json<UpdateLeadStageRequest>,
) -> Result<Json<Lead>> {
    let tenant_id_str = claims.tenant_id.clone();
    let tenant_id = Uuid::parse_str(&tenant_id_str)?;

    let probability = calculate_probability(&payload.stage);

    let lead = sqlx::query_as!(
        Lead,
        r#"
        UPDATE crm_leads
        SET
            stage = $1::lead_stage,
            probability = $2,
            lost_reason = $3,
            actual_close_date = CASE
                WHEN $1::text IN ('won', 'lost') THEN CURRENT_DATE
                ELSE actual_close_date
            END,
            updated_at = NOW()
        WHERE id = $4 AND tenant_id = $5
        RETURNING
            id, tenant_id, name, company, email, phone,
            source as "source: LeadSource",
            stage as "stage: LeadStage",
            score, value, probability, owner_id,
            expected_close_date, actual_close_date, lost_reason,
            custom_fields as "custom_fields: serde_json::Value",
            created_at, updated_at
        "#,
        payload.stage as LeadStage,
        probability,
        payload.reason,
        lead_id,
        tenant_id,
    )
    .fetch_optional(&state.pool)
    .await?
    .ok_or_else(|| AppError::not_found("Lead"))?;

    CacheInvalidation::on_lead_modified(&state.cache, &tenant_id_str, &lead_id.to_string()).await?;

    tracing::info!(tenant = %tenant_id, lead_id = %lead_id, new_stage = ?payload.stage, "Lead stage updated");

    Ok(Json(lead))
}

async fn get_lead_history(
    claims: Claims,
    State(state): State<Arc<CrmState>>,
    Path(lead_id): Path<Uuid>,
) -> Result<Json<Vec<LeadStageHistory>>> {
    let tenant_id = Uuid::parse_str(&claims.tenant_id)?;

    let _ = sqlx::query!(
        "SELECT id FROM crm_leads WHERE id = $1 AND tenant_id = $2",
        lead_id,
        tenant_id
    )
    .fetch_optional(&state.pool)
    .await?
    .ok_or_else(|| AppError::not_found("Lead"))?;

    let history = sqlx::query_as!(
        LeadStageHistory,
        r#"
        SELECT
            id, lead_id,
            from_stage as "from_stage: Option<LeadStage>",
            to_stage as "to_stage: LeadStage",
            reason, changed_by, duration_seconds, changed_at
        FROM crm_lead_stage_history
        WHERE lead_id = $1
        ORDER BY changed_at DESC
        "#,
        lead_id
    )
    .fetch_all(&state.pool)
    .await?;

    Ok(Json(history))
}

async fn sales_forecast(
    claims: Claims,
    State(state): State<Arc<CrmState>>,
) -> Result<Json<ForecastResponse>> {
    let tenant_id_str = claims.tenant_id.clone();
    let tenant_id = Uuid::parse_str(&tenant_id_str)?;

    if let Some(cached) = state
        .cache
        .get::<ForecastResponse>(&CacheKeys::forecast(&tenant_id_str))
        .await?
    {
        return Ok(Json(cached));
    }

    let result = sqlx::query!(
        r#"
        SELECT
            stage as "stage: LeadStage",
            COUNT(*) as "count!",
            COALESCE(SUM(value), 0) as "total_value!: Decimal",
            COALESCE(AVG(probability), 0) as "avg_probability!"
        FROM crm_leads
        WHERE tenant_id = $1
          AND stage NOT IN ('won', 'lost')
        GROUP BY stage
        "#,
        tenant_id
    )
    .fetch_all(&state.pool)
    .await?;

    let mut weighted_total = 0.0;
    let mut breakdown = serde_json::Map::new();

    for row in result {
        let total = row.total_value.to_f64().unwrap_or(0.0);
        let probability = row.avg_probability;
        let stage_weighted = total * (probability / 100.0);
        weighted_total += stage_weighted;

        breakdown.insert(
            format!("{:?}", row.stage).to_lowercase(),
            serde_json::json!({
                "count": row.count,
                "value": total,
                "probability": probability
            }),
        );
    }

    let forecast = ForecastResponse {
        period: Utc::now().format("%Y-%m").to_string(),
        forecast: ForecastValues {
            best_case: weighted_total * 1.35,
            most_likely: weighted_total,
            worst_case: weighted_total * 0.65,
            weighted: weighted_total,
        },
        breakdown_by_stage: serde_json::Value::Object(breakdown),
    };

    state
        .cache
        .set(&CacheKeys::forecast(&tenant_id_str), &forecast, CacheTTL::forecast())
        .await?;

    Ok(Json(forecast))
}

async fn pipeline_view(
    claims: Claims,
    State(state): State<Arc<CrmState>>,
) -> Result<Json<serde_json::Value>> {
    let tenant_id = Uuid::parse_str(&claims.tenant_id)?;

    let leads = sqlx::query_as!(
        Lead,
        r#"
        SELECT
            id, tenant_id, name, company, email, phone,
            source as "source: LeadSource",
            stage as "stage: LeadStage",
            score, value, probability, owner_id,
            expected_close_date, actual_close_date, lost_reason,
            custom_fields as "custom_fields: serde_json::Value",
            created_at, updated_at
        FROM crm_leads
        WHERE tenant_id = $1
          AND stage NOT IN ('won', 'lost')
        ORDER BY score DESC
        "#,
        tenant_id
    )
    .fetch_all(&state.pool)
    .await?;

    let mut pipeline = serde_json::Map::new();
    for stage in [
        LeadStage::New,
        LeadStage::Contacted,
        LeadStage::Qualification,
        LeadStage::Proposal,
        LeadStage::Negotiation,
    ] {
        let stage_leads: Vec<_> = leads
            .iter()
            .filter(|lead| lead.stage == stage)
            .cloned()
            .collect();

        let total_value: f64 = stage_leads
            .iter()
            .map(|lead| lead.value.to_f64().unwrap_or(0.0))
            .sum();

        pipeline.insert(
            format!("{:?}", stage).to_lowercase(),
            serde_json::json!({
                "leads": stage_leads,
                "count": stage_leads.len(),
                "total_value": total_value
            }),
        );
    }

    Ok(Json(serde_json::Value::Object(pipeline)))
}

async fn create_account(
    _claims: Claims,
    State(_state): State<Arc<CrmState>>,
    Json(_payload): Json<serde_json::Value>,
) -> Result<(StatusCode, Json<Account>)> {
    Err(AppError::BadRequest("Not implemented yet".to_string()))
}

async fn list_accounts(
    claims: Claims,
    State(state): State<Arc<CrmState>>,
) -> Result<Json<Vec<Account>>> {
    let tenant_id = Uuid::parse_str(&claims.tenant_id)?;

    let accounts = sqlx::query_as!(
        Account,
        r#"
        SELECT
            id, tenant_id, name, cnpj, industry, employees_count, annual_revenue,
            website, address as "address: Option<serde_json::Value>",
            account_manager_id, health_score, status,
            custom_fields as "custom_fields: serde_json::Value",
            created_at, updated_at
        FROM crm_accounts
        WHERE tenant_id = $1
        ORDER BY name
        "#,
        tenant_id
    )
    .fetch_all(&state.pool)
    .await?;

    Ok(Json(accounts))
}

async fn get_account(
    claims: Claims,
    State(state): State<Arc<CrmState>>,
    Path(id): Path<Uuid>,
) -> Result<Json<Account>> {
    let tenant_id = Uuid::parse_str(&claims.tenant_id)?;

    let account = sqlx::query_as!(
        Account,
        r#"
        SELECT
            id, tenant_id, name, cnpj, industry, employees_count, annual_revenue,
            website, address as "address: Option<serde_json::Value>",
            account_manager_id, health_score, status,
            custom_fields as "custom_fields: serde_json::Value",
            created_at, updated_at
        FROM crm_accounts
        WHERE id = $1 AND tenant_id = $2
        "#,
        id,
        tenant_id
    )
    .fetch_optional(&state.pool)
    .await?
    .ok_or_else(|| AppError::not_found("Account"))?;

    Ok(Json(account))
}

async fn account_health_score(
    claims: Claims,
    State(state): State<Arc<CrmState>>,
    Path(account_id): Path<Uuid>,
) -> Result<Json<AccountHealthScore>> {
    let tenant_id = Uuid::parse_str(&claims.tenant_id)?;

    let account = sqlx::query_as!(
        Account,
        r#"
        SELECT
            id, tenant_id, name, cnpj, industry, employees_count, annual_revenue,
            website, address as "address: Option<serde_json::Value>",
            account_manager_id, health_score, status,
            custom_fields as "custom_fields: serde_json::Value",
            created_at, updated_at
        FROM crm_accounts
        WHERE id = $1 AND tenant_id = $2
        "#,
        account_id,
        tenant_id
    )
    .fetch_optional(&state.pool)
    .await?
    .ok_or_else(|| AppError::not_found("Account"))?;

    let base_score = account.health_score.unwrap_or(75);
    let status = if base_score >= 70 {
        "healthy"
    } else if base_score >= 50 {
        "at_risk"
    } else {
        "critical"
    };

    let response = AccountHealthScore {
        account_id: account.id,
        health_score: base_score,
        status: status.to_string(),
        factors: HealthFactors {
            engagement: FactorScore {
                score: 85,
                weight: 30,
                trend: "up".to_string(),
            },
            financial: FactorScore {
                score: 70,
                weight: 25,
                trend: "stable".to_string(),
            },
            support_tickets: FactorScore {
                score: 80,
                weight: 20,
                trend: "up".to_string(),
            },
            product_usage: FactorScore {
                score: 75,
                weight: 25,
                trend: "down".to_string(),
            },
        },
        risk_indicators: vec![RiskIndicator {
            r#type: "low_usage".to_string(),
            severity: "medium".to_string(),
            message: "Uso do produto caiu 15% no último mês".to_string(),
        }],
        recommendations: vec![
            "Agendar quarterly business review".to_string(),
            "Oferecer treinamento avançado".to_string(),
        ],
    };

    Ok(Json(response))
}

async fn create_activity(
    claims: Claims,
    State(state): State<Arc<CrmState>>,
    Json(payload): Json<CreateActivityRequest>,
) -> Result<(StatusCode, Json<Activity>)> {
    payload
        .validate()
        .map_err(|e| AppError::validation_error(e.to_string()))?;

    let CreateActivityRequest {
        r#type,
        subject,
        description,
        status,
        scheduled_at,
        duration_minutes,
        lead_id,
        account_id,
        contact_id,
        owner_id,
        attendees,
        custom_fields,
    } = payload;

    if lead_id.is_none() && account_id.is_none() && contact_id.is_none() {
        return Err(AppError::validation_error(
            "At least one of lead_id, account_id or contact_id must be provided",
        ));
    }

    let tenant_id = Uuid::parse_str(&claims.tenant_id)?;
    let owner_id = owner_id.unwrap_or(Uuid::parse_str(&claims.sub)?);
    let activity_id = Uuid::new_v4();
    let status = status.unwrap_or(ActivityStatus::Scheduled);
    let completed_at = matches!(status, ActivityStatus::Completed).then(Utc::now);

    let activity = sqlx::query_as!(
        Activity,
        r#"
        INSERT INTO crm_activities
            (id, tenant_id, type, subject, description, status, scheduled_at, completed_at,
             duration_minutes, lead_id, account_id, contact_id, owner_id, attendees, custom_fields)
        VALUES
            ($1, $2, $3::activity_type, $4, $5, $6::activity_status, $7, $8,
             $9, $10, $11, $12, $13, $14, $15)
        RETURNING
            id,
            tenant_id,
            type as "type: ActivityType",
            subject,
            description,
            status as "status: ActivityStatus",
            scheduled_at,
            completed_at,
            duration_minutes,
            lead_id,
            account_id,
            contact_id,
            owner_id,
            attendees as "attendees: Vec<Uuid>",
            custom_fields as "custom_fields: serde_json::Value",
            created_at
        "#,
        activity_id,
        tenant_id,
        r#type as ActivityType,
        subject,
        description,
        status as ActivityStatus,
        scheduled_at,
        completed_at,
        duration_minutes,
        lead_id,
        account_id,
        contact_id,
        owner_id,
        &attendees,
        custom_fields,
    )
    .fetch_one(&state.pool)
    .await?;

    tracing::info!(tenant = %tenant_id, activity_id = %activity_id, "Activity created");

    Ok((StatusCode::CREATED, Json(activity)))
}

async fn list_activities(
    claims: Claims,
    State(state): State<Arc<CrmState>>,
    Query(query): Query<ListActivitiesQuery>,
) -> Result<Json<PaginatedResponse<Activity>>> {
    let tenant_id = Uuid::parse_str(&claims.tenant_id)?;

    let ListActivitiesQuery {
        status,
        r#type,
        owner_id,
        lead_id,
        account_id,
        contact_id,
        from,
        to,
        page,
        limit,
    } = query;

    let page = page.unwrap_or(1).max(1);
    let limit = limit.unwrap_or(50).min(100);
    let offset = (page - 1) * limit;

    let total_items = sqlx::query_scalar!(
        r#"
        SELECT COUNT(*)
        FROM crm_activities
        WHERE tenant_id = $1
          AND ($2::activity_status IS NULL OR status = $2::activity_status)
          AND ($3::activity_type IS NULL OR type = $3::activity_type)
          AND ($4::uuid IS NULL OR owner_id = $4)
          AND ($5::uuid IS NULL OR lead_id = $5)
          AND ($6::uuid IS NULL OR account_id = $6)
          AND ($7::uuid IS NULL OR contact_id = $7)
          AND ($8::timestamptz IS NULL OR scheduled_at >= $8)
          AND ($9::timestamptz IS NULL OR scheduled_at <= $9)
        "#,
        tenant_id,
        status.clone() as Option<ActivityStatus>,
        r#type.clone() as Option<ActivityType>,
        owner_id,
        lead_id,
        account_id,
        contact_id,
        from,
        to,
    )
    .fetch_one(&state.pool)
    .await?;

    let activities = sqlx::query_as!(
        Activity,
        r#"
        SELECT
            id,
            tenant_id,
            type as "type: ActivityType",
            subject,
            description,
            status as "status: ActivityStatus",
            scheduled_at,
            completed_at,
            duration_minutes,
            lead_id,
            account_id,
            contact_id,
            owner_id,
            attendees as "attendees: Vec<Uuid>",
            custom_fields as "custom_fields: serde_json::Value",
            created_at
        FROM crm_activities
        WHERE tenant_id = $1
          AND ($2::activity_status IS NULL OR status = $2::activity_status)
          AND ($3::activity_type IS NULL OR type = $3::activity_type)
          AND ($4::uuid IS NULL OR owner_id = $4)
          AND ($5::uuid IS NULL OR lead_id = $5)
          AND ($6::uuid IS NULL OR account_id = $6)
          AND ($7::uuid IS NULL OR contact_id = $7)
          AND ($8::timestamptz IS NULL OR scheduled_at >= $8)
          AND ($9::timestamptz IS NULL OR scheduled_at <= $9)
        ORDER BY scheduled_at NULLS LAST, created_at DESC
        LIMIT $10 OFFSET $11
        "#,
        tenant_id,
        status as Option<ActivityStatus>,
        r#type as Option<ActivityType>,
        owner_id,
        lead_id,
        account_id,
        contact_id,
        from,
        to,
        limit as i64,
        offset as i64,
    )
    .fetch_all(&state.pool)
    .await?;

    Ok(Json(PaginatedResponse::new(activities, page, limit, total_items)))
}

async fn get_activity(
    claims: Claims,
    State(state): State<Arc<CrmState>>,
    Path(id): Path<Uuid>,
) -> Result<Json<Activity>> {
    let tenant_id = Uuid::parse_str(&claims.tenant_id)?;

    let activity = sqlx::query_as!(
        Activity,
        r#"
        SELECT
            id,
            tenant_id,
            type as "type: ActivityType",
            subject,
            description,
            status as "status: ActivityStatus",
            scheduled_at,
            completed_at,
            duration_minutes,
            lead_id,
            account_id,
            contact_id,
            owner_id,
            attendees as "attendees: Vec<Uuid>",
            custom_fields as "custom_fields: serde_json::Value",
            created_at
        FROM crm_activities
        WHERE id = $1 AND tenant_id = $2
        "#,
        id,
        tenant_id,
    )
    .fetch_optional(&state.pool)
    .await?
    .ok_or_else(|| AppError::not_found("Activity"))?;

    Ok(Json(activity))
}

async fn update_activity(
    claims: Claims,
    State(state): State<Arc<CrmState>>,
    Path(id): Path<Uuid>,
    Json(payload): Json<UpdateActivityRequest>,
) -> Result<Json<Activity>> {
    payload
        .validate()
        .map_err(|e| AppError::validation_error(e.to_string()))?;

    let UpdateActivityRequest {
        subject,
        description,
        status,
        scheduled_at,
        completed_at,
        duration_minutes,
        lead_id,
        account_id,
        contact_id,
        owner_id,
        r#type,
        attendees,
        custom_fields,
    } = payload;

    let tenant_id = Uuid::parse_str(&claims.tenant_id)?;

    let mut activity = sqlx::query_as!(
        Activity,
        r#"
        SELECT
            id,
            tenant_id,
            type as "type: ActivityType",
            subject,
            description,
            status as "status: ActivityStatus",
            scheduled_at,
            completed_at,
            duration_minutes,
            lead_id,
            account_id,
            contact_id,
            owner_id,
            attendees as "attendees: Vec<Uuid>",
            custom_fields as "custom_fields: serde_json::Value",
            created_at
        FROM crm_activities
        WHERE id = $1 AND tenant_id = $2
        "#,
        id,
        tenant_id,
    )
    .fetch_optional(&state.pool)
    .await?
    .ok_or_else(|| AppError::not_found("Activity"))?;

    if let Some(subject) = subject {
        activity.subject = subject;
    }

    if let Some(description) = description {
        activity.description = Some(description);
    }

    if let Some(activity_type) = r#type {
        activity.r#type = activity_type;
    }

    if let Some(scheduled_at) = scheduled_at {
        activity.scheduled_at = Some(scheduled_at);
    }

    if let Some(duration) = duration_minutes {
        activity.duration_minutes = Some(duration);
    }

    if let Some(new_status) = status {
        activity.status = new_status;
        if new_status == ActivityStatus::Completed {
            activity.completed_at = Some(completed_at.unwrap_or_else(Utc::now));
        }
    } else if let Some(completed_at) = completed_at {
        activity.completed_at = Some(completed_at);
    }

    if let Some(lead_id) = lead_id {
        activity.lead_id = Some(lead_id);
    }

    if let Some(account_id) = account_id {
        activity.account_id = Some(account_id);
    }

    if let Some(contact_id) = contact_id {
        activity.contact_id = Some(contact_id);
    }

    if let Some(owner_id) = owner_id {
        activity.owner_id = owner_id;
    }

    if let Some(attendees) = attendees {
        activity.attendees = attendees;
    }

    if let Some(custom_fields) = custom_fields {
        activity.custom_fields = custom_fields;
    }

    let updated = sqlx::query_as!(
        Activity,
        r#"
        UPDATE crm_activities
        SET
            subject = $1,
            description = $2,
            status = $3::activity_status,
            scheduled_at = $4,
            completed_at = $5,
            duration_minutes = $6,
            lead_id = $7,
            account_id = $8,
            contact_id = $9,
            owner_id = $10,
            type = $11::activity_type,
            attendees = $12,
            custom_fields = $13,
            updated_at = NOW()
        WHERE id = $14 AND tenant_id = $15
        RETURNING
            id,
            tenant_id,
            type as "type: ActivityType",
            subject,
            description,
            status as "status: ActivityStatus",
            scheduled_at,
            completed_at,
            duration_minutes,
            lead_id,
            account_id,
            contact_id,
            owner_id,
            attendees as "attendees: Vec<Uuid>",
            custom_fields as "custom_fields: serde_json::Value",
            created_at
        "#,
        activity.subject,
        activity.description,
        activity.status as ActivityStatus,
        activity.scheduled_at,
        activity.completed_at,
        activity.duration_minutes,
        activity.lead_id,
        activity.account_id,
        activity.contact_id,
        activity.owner_id,
        activity.r#type as ActivityType,
        &activity.attendees,
        activity.custom_fields,
        id,
        tenant_id,
    )
    .fetch_one(&state.pool)
    .await?;

    tracing::info!(tenant = %tenant_id, activity_id = %id, "Activity updated");

    Ok(Json(updated))
}

async fn create_contact(
    claims: Claims,
    State(state): State<Arc<CrmState>>,
    Json(payload): Json<CreateContactRequest>,
) -> Result<(StatusCode, Json<Contact>)> {
    payload
        .validate()
        .map_err(|e| AppError::validation_error(e.to_string()))?;

    if payload.email.is_none() && payload.phone.is_none() {
        return Err(AppError::validation_error(
            "Informe ao menos email ou phone para o contato",
        ));
    }

    let tenant_id = Uuid::parse_str(&claims.tenant_id)?;

    if let Some(account_id) = payload.account_id {
        let exists = sqlx::query_scalar!(
            r#"
            SELECT EXISTS (
                SELECT 1 FROM crm_accounts
                WHERE id = $1 AND tenant_id = $2
            )
            "#,
            account_id,
            tenant_id,
        )
        .fetch_one(&state.pool)
        .await?;

        if !exists.unwrap_or(false) {
            return Err(AppError::validation_error(
                "Conta associada não encontrada para este tenant",
            ));
        }
    }

    let CreateContactRequest {
        account_id,
        name,
        email,
        phone,
        position,
        department,
        is_decision_maker,
        linkedin_url,
        custom_fields,
    } = payload;

    let contact_id = Uuid::new_v4();

    let contact = sqlx::query_as!(
        Contact,
        r#"
        INSERT INTO crm_contacts
            (id, tenant_id, account_id, name, email, phone, position, department,
             is_decision_maker, linkedin_url, custom_fields)
        VALUES
            ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11)
        RETURNING
            id,
            tenant_id,
            account_id,
            name,
            email,
            phone,
            position,
            department,
            is_decision_maker,
            linkedin_url,
            custom_fields as "custom_fields: serde_json::Value",
            created_at,
            updated_at
        "#,
        contact_id,
        tenant_id,
        account_id,
        name,
        email,
        phone,
        position,
        department,
        is_decision_maker,
        linkedin_url,
        custom_fields,
    )
    .fetch_one(&state.pool)
    .await?;

    tracing::info!(tenant = %tenant_id, contact_id = %contact_id, "Contact created");

    Ok((StatusCode::CREATED, Json(contact)))
}

async fn list_contacts(
    claims: Claims,
    State(state): State<Arc<CrmState>>,
    Query(query): Query<ListContactsQuery>,
) -> Result<Json<PaginatedResponse<Contact>>> {
    let tenant_id = Uuid::parse_str(&claims.tenant_id)?;

    let ListContactsQuery {
        account_id,
        is_decision_maker,
        department,
        search,
        page,
        limit,
    } = query;

    let department = department.as_deref();
    let search = search.as_deref();

    let page = page.unwrap_or(1).max(1);
    let limit = limit.unwrap_or(50).min(100);
    let offset = (page - 1) * limit;

    let total_items = sqlx::query_scalar!(
        r#"
        SELECT COUNT(*)
        FROM crm_contacts
        WHERE tenant_id = $1
          AND ($2::uuid IS NULL OR account_id = $2)
          AND ($3::bool IS NULL OR is_decision_maker = $3)
          AND ($4::text IS NULL OR department ILIKE '%' || $4 || '%')
          AND (
                $5::text IS NULL OR (
                    name ILIKE '%' || $5 || '%'
                    OR email ILIKE '%' || $5 || '%'
                )
          )
        "#,
        tenant_id,
        account_id,
        is_decision_maker,
        department,
        search,
    )
    .fetch_one(&state.pool)
    .await?;

    let contacts = sqlx::query_as!(
        Contact,
        r#"
        SELECT
            id,
            tenant_id,
            account_id,
            name,
            email,
            phone,
            position,
            department,
            is_decision_maker,
            linkedin_url,
            custom_fields as "custom_fields: serde_json::Value",
            created_at,
            updated_at
        FROM crm_contacts
        WHERE tenant_id = $1
          AND ($2::uuid IS NULL OR account_id = $2)
          AND ($3::bool IS NULL OR is_decision_maker = $3)
          AND ($4::text IS NULL OR department ILIKE '%' || $4 || '%')
          AND (
                $5::text IS NULL OR (
                    name ILIKE '%' || $5 || '%'
                    OR email ILIKE '%' || $5 || '%'
                )
          )
        ORDER BY name
        LIMIT $6 OFFSET $7
        "#,
        tenant_id,
        account_id,
        is_decision_maker,
        department,
        search,
        limit as i64,
        offset as i64,
    )
    .fetch_all(&state.pool)
    .await?;

    Ok(Json(PaginatedResponse::new(contacts, page, limit, total_items)))
}

async fn get_contact(
    claims: Claims,
    State(state): State<Arc<CrmState>>,
    Path(id): Path<Uuid>,
) -> Result<Json<Contact>> {
    let tenant_id = Uuid::parse_str(&claims.tenant_id)?;

    let contact = sqlx::query_as!(
        Contact,
        r#"
        SELECT
            id,
            tenant_id,
            account_id,
            name,
            email,
            phone,
            position,
            department,
            is_decision_maker,
            linkedin_url,
            custom_fields as "custom_fields: serde_json::Value",
            created_at,
            updated_at
        FROM crm_contacts
        WHERE id = $1 AND tenant_id = $2
        "#,
        id,
        tenant_id,
    )
    .fetch_optional(&state.pool)
    .await?
    .ok_or_else(|| AppError::not_found("Contact"))?;

    Ok(Json(contact))
}

async fn update_contact(
    claims: Claims,
    State(state): State<Arc<CrmState>>,
    Path(id): Path<Uuid>,
    Json(payload): Json<UpdateContactRequest>,
) -> Result<Json<Contact>> {
    payload
        .validate()
        .map_err(|e| AppError::validation_error(e.to_string()))?;

    let tenant_id = Uuid::parse_str(&claims.tenant_id)?;

    let mut contact = sqlx::query_as!(
        Contact,
        r#"
        SELECT
            id,
            tenant_id,
            account_id,
            name,
            email,
            phone,
            position,
            department,
            is_decision_maker,
            linkedin_url,
            custom_fields as "custom_fields: serde_json::Value",
            created_at,
            updated_at
        FROM crm_contacts
        WHERE id = $1 AND tenant_id = $2
        "#,
        id,
        tenant_id,
    )
    .fetch_optional(&state.pool)
    .await?
    .ok_or_else(|| AppError::not_found("Contact"))?;

    if payload.clear_account {
        contact.account_id = None;
    }
    if let Some(account_id) = payload.account_id {
        contact.account_id = Some(account_id);
    }

    if let Some(name) = payload.name {
        contact.name = name;
    }

    if payload.clear_email {
        contact.email = None;
    }
    if let Some(email) = payload.email {
        contact.email = Some(email);
    }

    if payload.clear_phone {
        contact.phone = None;
    }
    if let Some(phone) = payload.phone {
        contact.phone = Some(phone);
    }

    if payload.clear_position {
        contact.position = None;
    }
    if let Some(position) = payload.position {
        contact.position = Some(position);
    }

    if payload.clear_department {
        contact.department = None;
    }
    if let Some(department) = payload.department {
        contact.department = Some(department);
    }

    if let Some(is_decision_maker) = payload.is_decision_maker {
        contact.is_decision_maker = is_decision_maker;
    }

    if payload.clear_linkedin {
        contact.linkedin_url = None;
    }
    if let Some(linkedin) = payload.linkedin_url {
        contact.linkedin_url = Some(linkedin);
    }

    if let Some(custom_fields) = payload.custom_fields {
        contact.custom_fields = custom_fields;
    }

    let updated = sqlx::query_as!(
        Contact,
        r#"
        UPDATE crm_contacts
        SET
            account_id = $1,
            name = $2,
            email = $3,
            phone = $4,
            position = $5,
            department = $6,
            is_decision_maker = $7,
            linkedin_url = $8,
            custom_fields = $9,
            updated_at = NOW()
        WHERE id = $10 AND tenant_id = $11
        RETURNING
            id,
            tenant_id,
            account_id,
            name,
            email,
            phone,
            position,
            department,
            is_decision_maker,
            linkedin_url,
            custom_fields as "custom_fields: serde_json::Value",
            created_at,
            updated_at
        "#,
        contact.account_id,
        contact.name,
        contact.email,
        contact.phone,
        contact.position,
        contact.department,
        contact.is_decision_maker,
        contact.linkedin_url,
        contact.custom_fields,
        id,
        tenant_id,
    )
    .fetch_one(&state.pool)
    .await?;

    tracing::info!(tenant = %tenant_id, contact_id = %id, "Contact updated");

    Ok(Json(updated))
}

// ============================================================================
// HELPERS
// ============================================================================

fn calculate_lead_score(lead: &CreateLeadRequest) -> i32 {
    let mut score = 50;

    if lead.value > 100_000.0 {
        score += 20;
    } else if lead.value > 50_000.0 {
        score += 10;
    }

    score += match lead.source {
        LeadSource::LinkedIn => 15,
        LeadSource::Referral => 20,
        LeadSource::Website => 10,
        _ => 5,
    };

    if lead.company.is_some() {
        score += 5;
    }

    score.min(100)
}

fn calculate_probability(stage: &LeadStage) -> i32 {
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

async fn get_user_summary(pool: &DbPool, user_id: Uuid) -> Result<UserSummary> {
    let user = sqlx::query!(
        "SELECT id, name, email FROM users WHERE id = $1",
        user_id
    )
    .fetch_optional(pool)
    .await?
    .ok_or_else(|| AppError::not_found("User"))?;

    Ok(UserSummary {
        id: user.id,
        name: user.name,
        email: user.email,
    })
}

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
            source: LeadSource::LinkedIn,
            value: 150_000.0,
            expected_close_date: None,
            custom_fields: serde_json::json!({}),
        };

        let score = calculate_lead_score(&lead);
        assert!(score >= 80);
    }

    #[test]
    fn test_calculate_probability() {
        assert_eq!(calculate_probability(&LeadStage::Qualification), 25);
        assert_eq!(calculate_probability(&LeadStage::Proposal), 60);
        assert_eq!(calculate_probability(&LeadStage::Won), 100);
    }
}
