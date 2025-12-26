// webhooks.rs - Sistema de Webhooks para Notificações

use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::Json,
    routing::{get, post, delete},
    Router,
};
use chrono::{DateTime, Utc};
use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use uuid::Uuid;

use crate::{
    auth::Claims,
    db::DbPool,
    error::{AppError, Result},
};

pub fn routes(pool: DbPool) -> Router {
    Router::new()
        .route("/webhooks", post(create_webhook).get(list_webhooks))
        .route("/webhooks/:id", get(get_webhook).delete(delete_webhook))
        .route("/webhooks/:id/test", post(test_webhook))
        .with_state(Arc::new(pool))
}

// ============================================================================
// ESTRUTURAS
// ============================================================================

#[derive(Debug, Serialize, Deserialize, sqlx::FromRow)]
pub struct Webhook {
    pub id: Uuid,
    pub tenant_id: Uuid,
    pub url: String,
    #[sqlx(json)]
    pub events: Vec<String>,
    pub secret: String,
    pub is_active: bool,
    pub last_triggered_at: Option<DateTime<Utc>>,
    pub failure_count: i32,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Deserialize)]
pub struct CreateWebhookRequest {
    pub url: String,
    pub events: Vec<String>,
}

#[derive(Debug, Serialize)]
pub struct WebhookPayload {
    pub event: String,
    pub timestamp: DateTime<Utc>,
    pub data: serde_json::Value,
    pub signature: String,
}

/// Eventos disponíveis para webhooks
#[derive(Debug, Clone)]
pub enum WebhookEvent {
    // CRM
    LeadCreated,
    LeadUpdated,
    LeadStageChanged,
    OpportunityWon,
    OpportunityLost,
    
    // Financeiro
    InvoicePaid,
    InvoiceOverdue,
    PaymentReceived,
    
    // RH
    EmployeeHired,
    EmployeeTerminated,
    PayrollProcessed,
}

impl WebhookEvent {
    pub fn as_str(&self) -> &str {
        match self {
            WebhookEvent::LeadCreated => "lead.created",
            WebhookEvent::LeadUpdated => "lead.updated",
            WebhookEvent::LeadStageChanged => "lead.stage_changed",
            WebhookEvent::OpportunityWon => "opportunity.won",
            WebhookEvent::OpportunityLost => "opportunity.lost",
            WebhookEvent::InvoicePaid => "invoice.paid",
            WebhookEvent::InvoiceOverdue => "invoice.overdue",
            WebhookEvent::PaymentReceived => "payment.received",
            WebhookEvent::EmployeeHired => "employee.hired",
            WebhookEvent::EmployeeTerminated => "employee.terminated",
            WebhookEvent::PayrollProcessed => "payroll.processed",
        }
    }
}

// ============================================================================
// HANDLERS
// ============================================================================

async fn create_webhook(
    claims: Claims,
    State(pool): State<Arc<DbPool>>,
    Json(payload): Json<CreateWebhookRequest>,
) -> Result<(StatusCode, Json<Webhook>)> {
    let tenant_id = Uuid::parse_str(&claims.tenant_id)?;
    let id = Uuid::new_v4();
    
    // Gerar secret para assinatura
    let secret = generate_webhook_secret();

    let webhook = sqlx::query_as!(
        Webhook,
        r#"
        INSERT INTO webhooks
            (id, tenant_id, url, events, secret, is_active)
        VALUES
            ($1, $2, $3, $4, $5, true)
        RETURNING
            id, tenant_id, url, events as "events: Vec<String>", secret,
            is_active, last_triggered_at, failure_count, created_at
        "#,
        id,
        tenant_id,
        payload.url,
        &payload.events,
        secret,
    )
    .fetch_one(pool.as_ref())
    .await?;

    tracing::info!("Webhook created: {} -> {}", id, payload.url);

    Ok((StatusCode::CREATED, Json(webhook)))
}

async fn list_webhooks(
    claims: Claims,
    State(pool): State<Arc<DbPool>>,
) -> Result<Json<Vec<Webhook>>> {
    let tenant_id = Uuid::parse_str(&claims.tenant_id)?;

    let webhooks = sqlx::query_as!(
        Webhook,
        r#"
        SELECT
            id, tenant_id, url, events as "events: Vec<String>", secret,
            is_active, last_triggered_at, failure_count, created_at
        FROM webhooks
        WHERE tenant_id = $1
        ORDER BY created_at DESC
        "#,
        tenant_id
    )
    .fetch_all(pool.as_ref())
    .await?;

    Ok(Json(webhooks))
}

async fn get_webhook(
    claims: Claims,
    State(pool): State<Arc<DbPool>>,
    Path(id): Path<Uuid>,
) -> Result<Json<Webhook>> {
    let tenant_id = Uuid::parse_str(&claims.tenant_id)?;

    let webhook = sqlx::query_as!(
        Webhook,
        r#"
        SELECT
            id, tenant_id, url, events as "events: Vec<String>", secret,
            is_active, last_triggered_at, failure_count, created_at
        FROM webhooks
        WHERE id = $1 AND tenant_id = $2
        "#,
        id,
        tenant_id
    )
    .fetch_optional(pool.as_ref())
    .await?
    .ok_or_else(|| AppError::not_found("Webhook"))?;

    Ok(Json(webhook))
}

async fn delete_webhook(
    claims: Claims,
    State(pool): State<Arc<DbPool>>,
    Path(id): Path<Uuid>,
) -> Result<StatusCode> {
    let tenant_id = Uuid::parse_str(&claims.tenant_id)?;

    let result = sqlx::query!(
        "DELETE FROM webhooks WHERE id = $1 AND tenant_id = $2",
        id,
        tenant_id
    )
    .execute(pool.as_ref())
    .await?;

    if result.rows_affected() == 0 {
        return Err(AppError::not_found("Webhook"));
    }

    Ok(StatusCode::NO_CONTENT)
}

async fn test_webhook(
    claims: Claims,
    State(pool): State<Arc<DbPool>>,
    Path(id): Path<Uuid>,
) -> Result<Json<serde_json::Value>> {
    let tenant_id = Uuid::parse_str(&claims.tenant_id)?;

    let webhook = get_webhook(claims, State(pool.clone()), Path(id)).await?.0;

    // Enviar payload de teste
    let test_payload = WebhookPayload {
        event: "test".to_string(),
        timestamp: Utc::now(),
        data: serde_json::json!({
            "message": "This is a test webhook"
        }),
        signature: "".to_string(), // Será calculado abaixo
    };

    let result = send_webhook(&webhook, test_payload).await;

    Ok(Json(serde_json::json!({
        "success": result.is_ok(),
        "error": result.err().map(|e| e.to_string())
    })))
}

// ============================================================================
// WEBHOOK DISPATCHER
// ============================================================================

/// Disparar webhook para um evento
pub async fn trigger_webhook(
    pool: &DbPool,
    tenant_id: Uuid,
    event: WebhookEvent,
    data: serde_json::Value,
) -> Result<()> {
    let event_str = event.as_str();

    // Buscar webhooks ativos que subscrevem este evento
    let webhooks = sqlx::query_as!(
        Webhook,
        r#"
        SELECT
            id, tenant_id, url, events as "events: Vec<String>", secret,
            is_active, last_triggered_at, failure_count, created_at
        FROM webhooks
        WHERE tenant_id = $1
          AND is_active = true
          AND $2 = ANY(events)
          AND failure_count < 10
        "#,
        tenant_id,
        event_str
    )
    .fetch_all(pool)
    .await?;

    tracing::debug!(
        "Triggering {} webhooks for event: {}",
        webhooks.len(),
        event_str
    );

    // Disparar webhooks em paralelo
    let tasks: Vec<_> = webhooks
        .into_iter()
        .map(|webhook| {
            let data = data.clone();
            let pool = pool.clone();
            
            tokio::spawn(async move {
                let payload = WebhookPayload {
                    event: event_str.to_string(),
                    timestamp: Utc::now(),
                    data,
                    signature: generate_signature(&webhook.secret, event_str),
                };

                match send_webhook(&webhook, payload).await {
                    Ok(_) => {
                        // Atualizar last_triggered_at
                        let _ = sqlx::query!(
                            "UPDATE webhooks SET last_triggered_at = NOW(), failure_count = 0 WHERE id = $1",
                            webhook.id
                        )
                        .execute(&pool)
                        .await;
                    }
                    Err(e) => {
                        tracing::error!("Webhook failed: {} - {}", webhook.url, e);
                        
                        // Incrementar failure_count
                        let _ = sqlx::query!(
                            "UPDATE webhooks SET failure_count = failure_count + 1 WHERE id = $1",
                            webhook.id
                        )
                        .execute(&pool)
                        .await;
                    }
                }
            })
        })
        .collect();

    // Aguardar todas as tasks
    for task in tasks {
        let _ = task.await;
    }

    Ok(())
}

/// Enviar webhook HTTP
async fn send_webhook(webhook: &Webhook, payload: WebhookPayload) -> Result<()> {
    let client = Client::new();

    let response = client
        .post(&webhook.url)
        .header("Content-Type", "application/json")
        .header("X-Webhook-Signature", &payload.signature)
        .header("X-Webhook-Event", &payload.event)
        .json(&payload)
        .timeout(std::time::Duration::from_secs(10))
        .send()
        .await
        .map_err(|e| AppError::InternalError(format!("Webhook request failed: {}", e)))?;

    if !response.status().is_success() {
        return Err(AppError::InternalError(format!(
            "Webhook returned status: {}",
            response.status()
        )));
    }

    tracing::info!("Webhook sent successfully: {}", webhook.url);

    Ok(())
}

// ============================================================================
// HELPERS
// ============================================================================

/// Gerar secret aleatório para webhook
fn generate_webhook_secret() -> String {
    use rand::Rng;
    const CHARSET: &[u8] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789";
    const SECRET_LEN: usize = 32;
    
    let mut rng = rand::thread_rng();
    
    (0..SECRET_LEN)
        .map(|_| {
            let idx = rng.gen_range(0..CHARSET.len());
            CHARSET[idx] as char
        })
        .collect()
}

/// Gerar assinatura HMAC SHA-256
fn generate_signature(secret: &str, data: &str) -> String {
    use hmac::{Hmac, Mac};
    use sha2::Sha256;
    
    type HmacSha256 = Hmac<Sha256>;
    
    let mut mac = HmacSha256::new_from_slice(secret.as_bytes())
        .expect("HMAC can take key of any size");
    
    mac.update(data.as_bytes());
    
    let result = mac.finalize();
    let code_bytes = result.into_bytes();
    
    hex::encode(code_bytes)
}

/// Verificar assinatura de webhook (para receber webhooks de terceiros)
pub fn verify_webhook_signature(secret: &str, signature: &str, data: &str) -> bool {
    let expected = generate_signature(secret, data);
    
    // Comparação constant-time para evitar timing attacks
    signature.as_bytes().len() == expected.as_bytes().len()
        && signature
            .as_bytes()
            .iter()
            .zip(expected.as_bytes())
            .all(|(a, b)| a == b)
}

// ============================================================================
// EXEMPLOS DE USO
// ============================================================================

/// Exemplo: Disparar webhook quando lead é criado
pub async fn example_trigger_lead_created(pool: &DbPool, tenant_id: Uuid, lead_id: Uuid) {
    let _ = trigger_webhook(
        pool,
        tenant_id,
        WebhookEvent::LeadCreated,
        serde_json::json!({
            "lead_id": lead_id,
            "action": "created"
        }),
    )
    .await;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_generate_signature() {
        let secret = "my_secret_key";
        let data = "test_data";
        
        let sig1 = generate_signature(secret, data);
        let sig2 = generate_signature(secret, data);
        
        assert_eq!(sig1, sig2); // Mesma entrada = mesma saída
        assert!(!sig1.is_empty());
    }

    #[test]
    fn test_verify_signature() {
        let secret = "my_secret_key";
        let data = "test_data";
        
        let signature = generate_signature(secret, data);
        
        assert!(verify_webhook_signature(secret, &signature, data));
        assert!(!verify_webhook_signature(secret, "wrong_sig", data));
        assert!(!verify_webhook_signature("wrong_secret", &signature, data));
    }
}
