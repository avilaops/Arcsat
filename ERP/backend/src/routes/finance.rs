// routes/finance.rs - Módulo Financeiro Enterprise

use crate::{
    auth::Claims,
    db::DbPool,
    error::{AppError, Result},
    models::*,
};
use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    response::Json,
    routing::{get, post, patch},
    Router,
};
use rust_decimal::Decimal;
use std::sync::Arc;
use uuid::Uuid;
use validator::Validate;
use chrono::{Datelike, NaiveDate, Utc};

pub fn routes(pool: DbPool) -> Router {
    Router::new()
        // Contas a Pagar
        .route("/accounts-payable", post(create_accounts_payable).get(list_accounts_payable))
        .route("/accounts-payable/:id", get(get_accounts_payable).patch(update_accounts_payable))
        .route("/accounts-payable/:id/pay", post(pay_accounts_payable))
        
        // Contas a Receber
        .route("/accounts-receivable", post(create_accounts_receivable).get(list_accounts_receivable))
        .route("/accounts-receivable/:id", get(get_accounts_receivable))
        
        // Fluxo de Caixa
        .route("/cashflow/projection", post(cashflow_projection))
        .route("/cashflow/realtime", get(cashflow_realtime))
        
        // DRE
        .route("/dre/realtime", get(dre_realtime))
        .route("/dre/monthly", get(dre_monthly))
        
        // Pagamentos
        .route("/payments/pix", post(create_pix_payment))
        .route("/payments/:id/status", get(payment_status))
        
        // Invoices (NFe)
        .route("/invoices/nfe", post(issue_nfe))
        .route("/invoices/:id", get(get_invoice))
        
        .with_state(Arc::new(pool))
}

// ============================================================================
// CONTAS A PAGAR
// ============================================================================

async fn create_accounts_payable(
    claims: Claims,
    State(pool): State<Arc<DbPool>>,
    Json(payload): Json<CreateAccountsPayableRequest>,
) -> Result<(StatusCode, Json<AccountsPayable>)> {
    payload.validate()
        .map_err(|e| AppError::validation_error(e.to_string()))?;

    let id = Uuid::new_v4();
    let tenant_id = Uuid::parse_str(&claims.tenant_id)?;
    let created_by = Uuid::parse_str(&claims.sub)?;

    let ap = sqlx::query_as!(
        AccountsPayable,
        r#"
        INSERT INTO finance_accounts_payable
            (id, tenant_id, supplier_id, invoice_number, description, amount, due_date,
             status, category, cost_center, created_by)
        VALUES
            ($1, $2, $3, $4, $5, $6, $7, $8::payment_status, $9, $10, $11)
        RETURNING
            id, tenant_id, supplier_id, invoice_number, description, amount, due_date,
            payment_date, payment_method as "payment_method: Option<PaymentMethod>",
            status as "status: PaymentStatus", category, cost_center, notes, attachment_url,
            created_by, approved_by, created_at, updated_at
        "#,
        id,
        tenant_id,
        payload.supplier_id,
        payload.invoice_number,
        payload.description,
        Decimal::from_f64_retain(payload.amount).unwrap(),
        payload.due_date,
        PaymentStatus::Pending as PaymentStatus,
        payload.category,
        payload.cost_center,
        created_by,
    )
    .fetch_one(pool.as_ref())
    .await?;

    tracing::info!("Accounts payable created: {} - R$ {}", id, payload.amount);

    Ok((StatusCode::CREATED, Json(ap)))
}

async fn list_accounts_payable(
    claims: Claims,
    State(pool): State<Arc<DbPool>>,
    Query(query): Query<serde_json::Value>, // TODO: Criar struct de filtros
) -> Result<Json<Vec<AccountsPayable>>> {
    let tenant_id = Uuid::parse_str(&claims.tenant_id)?;

    let aps = sqlx::query_as!(
        AccountsPayable,
        r#"
        SELECT
            id, tenant_id, supplier_id, invoice_number, description, amount, due_date,
            payment_date, payment_method as "payment_method: Option<PaymentMethod>",
            status as "status: PaymentStatus", category, cost_center, notes, attachment_url,
            created_by, approved_by, created_at, updated_at
        FROM finance_accounts_payable
        WHERE tenant_id = $1
        ORDER BY due_date ASC
        "#,
        tenant_id
    )
    .fetch_all(pool.as_ref())
    .await?;

    Ok(Json(aps))
}

async fn get_accounts_payable(
    claims: Claims,
    State(pool): State<Arc<DbPool>>,
    Path(id): Path<Uuid>,
) -> Result<Json<AccountsPayable>> {
    let tenant_id = Uuid::parse_str(&claims.tenant_id)?;

    let ap = sqlx::query_as!(
        AccountsPayable,
        r#"
        SELECT
            id, tenant_id, supplier_id, invoice_number, description, amount, due_date,
            payment_date, payment_method as "payment_method: Option<PaymentMethod>",
            status as "status: PaymentStatus", category, cost_center, notes, attachment_url,
            created_by, approved_by, created_at, updated_at
        FROM finance_accounts_payable
        WHERE id = $1 AND tenant_id = $2
        "#,
        id,
        tenant_id
    )
    .fetch_optional(pool.as_ref())
    .await?
    .ok_or_else(|| AppError::not_found("Accounts Payable"))?;

    Ok(Json(ap))
}

async fn update_accounts_payable(
    claims: Claims,
    State(pool): State<Arc<DbPool>>,
    Path(id): Path<Uuid>,
    Json(payload): Json<CreateAccountsPayableRequest>,
) -> Result<Json<AccountsPayable>> {
    payload.validate()
        .map_err(|e| AppError::validation_error(e.to_string()))?;

    let tenant_id = Uuid::parse_str(&claims.tenant_id)?;

    let ap = sqlx::query_as!(
        AccountsPayable,
        r#"
        UPDATE finance_accounts_payable
        SET
            description = $1,
            amount = $2,
            due_date = $3,
            invoice_number = $4,
            supplier_id = $5,
            category = $6,
            cost_center = $7,
            updated_at = NOW()
        WHERE id = $8 AND tenant_id = $9
        RETURNING
            id, tenant_id, supplier_id, invoice_number, description, amount, due_date,
            payment_date, payment_method as "payment_method: Option<PaymentMethod>",
            status as "status: PaymentStatus", category, cost_center, notes, attachment_url,
            created_by, approved_by, created_at, updated_at
        "#,
        payload.description,
        Decimal::from_f64_retain(payload.amount).unwrap(),
        payload.due_date,
        payload.invoice_number,
        payload.supplier_id,
        payload.category,
        payload.cost_center,
        id,
        tenant_id
    )
    .fetch_optional(pool.as_ref())
    .await?
    .ok_or_else(|| AppError::not_found("Accounts Payable"))?;

    Ok(Json(ap))
}

async fn pay_accounts_payable(
    claims: Claims,
    State(pool): State<Arc<DbPool>>,
    Path(id): Path<Uuid>,
    Json(payload): Json<serde_json::Value>, // {payment_method, payment_date}
) -> Result<Json<AccountsPayable>> {
    let tenant_id = Uuid::parse_str(&claims.tenant_id)?;

    // TODO: Validar saldo, aprovar pagamento, registrar transação

    let ap = sqlx::query_as!(
        AccountsPayable,
        r#"
        UPDATE finance_accounts_payable
        SET
            status = $1::payment_status,
            payment_date = CURRENT_DATE,
            payment_method = $2::payment_method,
            updated_at = NOW()
        WHERE id = $3 AND tenant_id = $4
        RETURNING
            id, tenant_id, supplier_id, invoice_number, description, amount, due_date,
            payment_date, payment_method as "payment_method: Option<PaymentMethod>",
            status as "status: PaymentStatus", category, cost_center, notes, attachment_url,
            created_by, approved_by, created_at, updated_at
        "#,
        PaymentStatus::Paid as PaymentStatus,
        PaymentMethod::Pix as PaymentMethod, // TODO: Extrair do payload
        id,
        tenant_id
    )
    .fetch_optional(pool.as_ref())
    .await?
    .ok_or_else(|| AppError::not_found("Accounts Payable"))?;

    tracing::info!("Payment processed: {} - R$ {}", id, ap.amount);

    Ok(Json(ap))
}

// ============================================================================
// CONTAS A RECEBER
// ============================================================================

async fn create_accounts_receivable(
    claims: Claims,
    State(pool): State<Arc<DbPool>>,
    Json(payload): Json<serde_json::Value>,
) -> Result<(StatusCode, Json<AccountsReceivable>)> {
    // TODO: Implementar similar ao AP
    Err(AppError::BadRequest("Not implemented yet".to_string()))
}

async fn list_accounts_receivable(
    claims: Claims,
    State(pool): State<Arc<DbPool>>,
) -> Result<Json<Vec<AccountsReceivable>>> {
    let tenant_id = Uuid::parse_str(&claims.tenant_id)?;

    let ars = sqlx::query_as!(
        AccountsReceivable,
        r#"
        SELECT
            id, tenant_id, customer_id, invoice_number, description, amount, due_date,
            payment_date, payment_method as "payment_method: Option<PaymentMethod>",
            status as "status: PaymentStatus", notes, nfe_key, nfe_url,
            created_at, updated_at
        FROM finance_accounts_receivable
        WHERE tenant_id = $1
        ORDER BY due_date ASC
        "#,
        tenant_id
    )
    .fetch_all(pool.as_ref())
    .await?;

    Ok(Json(ars))
}

async fn get_accounts_receivable(
    claims: Claims,
    State(pool): State<Arc<DbPool>>,
    Path(id): Path<Uuid>,
) -> Result<Json<AccountsReceivable>> {
    let tenant_id = Uuid::parse_str(&claims.tenant_id)?;

    let ar = sqlx::query_as!(
        AccountsReceivable,
        r#"
        SELECT
            id, tenant_id, customer_id, invoice_number, description, amount, due_date,
            payment_date, payment_method as "payment_method: Option<PaymentMethod>",
            status as "status: PaymentStatus", notes, nfe_key, nfe_url,
            created_at, updated_at
        FROM finance_accounts_receivable
        WHERE id = $1 AND tenant_id = $2
        "#,
        id,
        tenant_id
    )
    .fetch_optional(pool.as_ref())
    .await?
    .ok_or_else(|| AppError::not_found("Accounts Receivable"))?;

    Ok(Json(ar))
}

// ============================================================================
// FLUXO DE CAIXA
// ============================================================================

#[derive(Debug, serde::Deserialize)]
struct CashflowProjectionRequest {
    start_date: NaiveDate,
    months: i32,
    #[serde(default)]
    scenario: String, // realistic, optimistic, pessimistic
    #[serde(default)]
    include_ai_prediction: bool,
}

async fn cashflow_projection(
    claims: Claims,
    State(pool): State<Arc<DbPool>>,
    Json(payload): Json<CashflowProjectionRequest>,
) -> Result<Json<Vec<CashflowProjection>>> {
    let tenant_id = Uuid::parse_str(&claims.tenant_id)?;

    // TODO: Implementar projeção real com ML
    // Por enquanto, retornar dados simulados

    let mut projections = Vec::new();
    let mut current_balance = 500000.0; // Saldo inicial (buscar do banco)

    for month in 0..payload.months {
        let current_date = payload.start_date + chrono::Duration::days(month as i64 * 30);
        
        // Simular fluxos (em produção, calcular com base em histórico + ML)
        let inflows = Inflows {
            sales: 350000.0 + (month as f64 * 5000.0),
            receivables: 120000.0,
            other: 10000.0,
        };

        let outflows = Outflows {
            payroll: 180000.0,
            suppliers: 95000.0,
            taxes: 45000.0,
            operational: 60000.0,
        };

        let total_inflow = inflows.sales + inflows.receivables + inflows.other;
        let total_outflow = outflows.payroll + outflows.suppliers + outflows.taxes + outflows.operational;
        
        let closing_balance = current_balance + total_inflow - total_outflow;

        projections.push(CashflowProjection {
            month: format!("{}-{:02}", current_date.year(), current_date.month()),
            opening_balance: current_balance,
            inflows,
            outflows,
            closing_balance,
            ai_confidence: 0.85 - (month as f32 * 0.05), // Confiança diminui com tempo
        });

        current_balance = closing_balance;
    }

    Ok(Json(projections))
}

async fn cashflow_realtime(
    claims: Claims,
    State(pool): State<Arc<DbPool>>,
) -> Result<Json<serde_json::Value>> {
    let tenant_id = Uuid::parse_str(&claims.tenant_id)?;

    // TODO: Calcular fluxo de caixa do mês atual em tempo real
    
    Ok(Json(serde_json::json!({
        "current_month": Utc::now().format("%Y-%m").to_string(),
        "opening_balance": 500000.0,
        "inflows_to_date": 280000.0,
        "outflows_to_date": 195000.0,
        "current_balance": 585000.0,
        "projected_closing": 620000.0
    })))
}

// ============================================================================
// DRE (DEMONSTRATIVO DE RESULTADOS)
// ============================================================================

async fn dre_realtime(
    claims: Claims,
    State(pool): State<Arc<DbPool>>,
    Query(query): Query<serde_json::Value>, // {start, end, group_by}
) -> Result<Json<DREResponse>> {
    let tenant_id = Uuid::parse_str(&claims.tenant_id)?;

    // TODO: Calcular DRE real baseado em transações
    // Por enquanto, dados simulados

    let response = DREResponse {
        period: "2024".to_string(),
        summary: DRESummary {
            revenue: RevenueMetrics {
                gross: 12500000.0,
                net: 11875000.0,
                growth_yoy: 23.5,
            },
            costs: CostMetrics {
                cogs: 4500000.0,
                gross_margin: 62.0,
            },
            expenses: ExpenseMetrics {
                sales: 2100000.0,
                administrative: 1850000.0,
                operational: 1200000.0,
            },
            ebitda: 2225000.0,
            ebitda_margin: 18.7,
            net_income: 1456000.0,
            net_margin: 12.3,
        },
        monthly: vec![], // TODO: Implementar breakdown mensal
    };

    Ok(Json(response))
}

async fn dre_monthly(
    claims: Claims,
    State(pool): State<Arc<DbPool>>,
    Query(query): Query<serde_json::Value>,
) -> Result<Json<Vec<MonthlyDRE>>> {
    let tenant_id = Uuid::parse_str(&claims.tenant_id)?;

    // TODO: Implementar DRE mensal
    
    Ok(Json(vec![]))
}

// ============================================================================
// PAGAMENTOS PIX
// ============================================================================

#[derive(Debug, serde::Deserialize)]
struct PixPaymentRequest {
    r#type: String, // pix_key, qr_code
    pix_key: Option<String>,
    amount: f64,
    description: String,
    scheduled_date: Option<NaiveDate>,
}

#[derive(Debug, serde::Serialize)]
struct PixPaymentResponse {
    id: Uuid,
    status: String,
    pix_qrcode: Option<String>,
    pix_copy_paste: Option<String>,
    expires_at: Option<chrono::DateTime<Utc>>,
}

async fn create_pix_payment(
    claims: Claims,
    State(pool): State<Arc<DbPool>>,
    Json(payload): Json<PixPaymentRequest>,
) -> Result<(StatusCode, Json<PixPaymentResponse>)> {
    let tenant_id = Uuid::parse_str(&claims.tenant_id)?;
    let payment_id = Uuid::new_v4();

    // TODO: Integrar com API do banco para gerar PIX
    // TODO: Salvar no banco de dados
    // TODO: Gerar QR Code

    let response = PixPaymentResponse {
        id: payment_id,
        status: "pending_approval".to_string(),
        pix_qrcode: Some("base64_encoded_qr_code".to_string()),
        pix_copy_paste: Some("00020126580014br.gov.bcb.pix...".to_string()),
        expires_at: Some(Utc::now() + chrono::Duration::hours(24)),
    };

    tracing::info!("PIX payment created: {} - R$ {}", payment_id, payload.amount);

    Ok((StatusCode::CREATED, Json(response)))
}

async fn payment_status(
    claims: Claims,
    State(pool): State<Arc<DbPool>>,
    Path(id): Path<Uuid>,
) -> Result<Json<serde_json::Value>> {
    let tenant_id = Uuid::parse_str(&claims.tenant_id)?;

    // TODO: Consultar status do pagamento no banco/API bancária

    Ok(Json(serde_json::json!({
        "id": id,
        "status": "paid",
        "paid_at": Utc::now()
    })))
}

// ============================================================================
// NOTAS FISCAIS (NFe)
// ============================================================================

#[derive(Debug, serde::Deserialize)]
struct IssueNFeRequest {
    customer: CustomerNFe,
    items: Vec<ItemNFe>,
    payment: PaymentNFe,
}

#[derive(Debug, serde::Deserialize)]
struct CustomerNFe {
    cnpj: String,
    name: String,
    address: serde_json::Value,
}

#[derive(Debug, serde::Deserialize)]
struct ItemNFe {
    code: String,
    description: String,
    quantity: f64,
    unit_price: f64,
    tax_rate: f64,
    ncm: String,
}

#[derive(Debug, serde::Deserialize)]
struct PaymentNFe {
    method: String,
    due_date: NaiveDate,
}

#[derive(Debug, serde::Serialize)]
struct NFeResponse {
    id: Uuid,
    number: String,
    series: String,
    access_key: String,
    status: String,
    xml_url: String,
    pdf_url: String,
    issued_at: chrono::DateTime<Utc>,
}

async fn issue_nfe(
    claims: Claims,
    State(pool): State<Arc<DbPool>>,
    Json(payload): Json<IssueNFeRequest>,
) -> Result<(StatusCode, Json<NFeResponse>)> {
    let tenant_id = Uuid::parse_str(&claims.tenant_id)?;
    let nfe_id = Uuid::new_v4();

    // TODO: Integrar com Sefaz/API de NFe
    // TODO: Validar dados
    // TODO: Gerar XML
    // TODO: Assinar digitalmente
    // TODO: Transmitir para SEFAZ
    // TODO: Salvar no banco

    let response = NFeResponse {
        id: nfe_id,
        number: "000123456".to_string(),
        series: "1".to_string(),
        access_key: "35240112345678000190550010001234561123456780".to_string(),
        status: "authorized".to_string(),
        xml_url: format!("https://storage.erp.com/{}/nfe-{}.xml", tenant_id, nfe_id),
        pdf_url: format!("https://storage.erp.com/{}/nfe-{}.pdf", tenant_id, nfe_id),
        issued_at: Utc::now(),
    };

    tracing::info!("NFe issued: {} - Access Key: {}", nfe_id, response.access_key);

    Ok((StatusCode::CREATED, Json(response)))
}

async fn get_invoice(
    claims: Claims,
    State(pool): State<Arc<DbPool>>,
    Path(id): Path<Uuid>,
) -> Result<Json<serde_json::Value>> {
    let tenant_id = Uuid::parse_str(&claims.tenant_id)?;

    // TODO: Buscar NFe do banco

    Ok(Json(serde_json::json!({
        "id": id,
        "number": "000123456",
        "status": "authorized"
    })))
}
