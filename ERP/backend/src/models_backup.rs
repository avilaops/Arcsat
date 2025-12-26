use chrono::{DateTime, NaiveDate, Utc};
use serde::{Deserialize, Serialize};
use sqlx::{FromRow, Type};
use uuid::Uuid;
use validator::Validate;
use rust_decimal::Decimal;

// ============================================================================
// CRM - LEADS
// ============================================================================

#[derive(Debug, Clone, Serialize, Deserialize, Type, PartialEq)]
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

#[derive(Debug, Clone, Serialize, Deserialize, Type)]
#[sqlx(type_name = "lead_source", rename_all = "lowercase")]
pub enum LeadSource {
    Website,
    LinkedIn,
    Referral,
    ColdCall,
    Event,
    Partner,
    Other,
}

#[derive(Debug, Serialize, Deserialize, FromRow, Clone)]
pub struct Lead {
    pub id: Uuid,
    pub tenant_id: Uuid,
    pub name: String,
    pub company: Option<String>,
    pub email: String,
    pub phone: Option<String>,
    #[sqlx(try_from = "String")]
    pub source: LeadSource,
    #[sqlx(try_from = "String")]
    pub stage: LeadStage,
    pub score: i32,
    pub value: Decimal,
    pub probability: i32,
    pub owner_id: Uuid,
    pub expected_close_date: Option<NaiveDate>,
    pub actual_close_date: Option<NaiveDate>,
    pub lost_reason: Option<String>,
    #[sqlx(json)]
    pub custom_fields: serde_json::Value,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Deserialize, Validate)]
pub struct CreateLeadRequest {
    #[validate(length(min = 2, max = 255))]
    pub name: String,

    #[validate(length(max = 255))]
    pub company: Option<String>,

    #[validate(email)]
    pub email: String,

    #[validate(length(max = 20))]
    pub phone: Option<String>,

    pub source: LeadSource,
    pub value: f64,
    pub expected_close_date: Option<NaiveDate>,

    #[serde(default)]
    pub custom_fields: serde_json::Value,
}

#[derive(Debug, Deserialize)]
pub struct UpdateLeadStageRequest {
    pub stage: LeadStage,
    pub reason: Option<String>,
    pub scheduled_call: Option<DateTime<Utc>>,
}

#[derive(Debug, Deserialize)]
pub struct ListLeadsQuery {
    pub stage: Option<LeadStage>,
    pub score_min: Option<i32>,
    pub owner_id: Option<Uuid>,
    pub source: Option<LeadSource>,
    pub page: Option<i32>,
    pub limit: Option<i32>,
}

#[derive(Debug, Serialize)]
pub struct LeadResponse {
    pub id: Uuid,
    pub name: String,
    pub company: Option<String>,
    pub email: String,
    pub phone: Option<String>,
    pub source: LeadSource,
    pub stage: LeadStage,
    pub score: i32,
    pub value: String,
    pub probability: i32,
    pub owner: UserSummary,
    pub created_at: DateTime<Utc>,
    pub last_contact: Option<DateTime<Utc>>,
}

#[derive(Debug, Serialize, Deserialize, FromRow)]
pub struct LeadStageHistory {
    pub id: Uuid,
    pub lead_id: Uuid,
    pub from_stage: Option<LeadStage>,
    pub to_stage: LeadStage,
    pub reason: Option<String>,
    pub changed_by: Option<Uuid>,
    pub duration_seconds: Option<i32>,
    pub changed_at: DateTime<Utc>,
}

// ============================================================================
// CRM - OPPORTUNITIES
// ============================================================================

#[derive(Debug, Serialize, Deserialize, FromRow)]
pub struct Opportunity {
    pub id: Uuid,
    pub tenant_id: Uuid,
    pub account_id: Uuid,
    pub name: String,
    pub stage: LeadStage,
    pub value: Decimal,
    pub probability: i32,
    pub expected_close_date: NaiveDate,
    pub owner_id: Uuid,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Serialize)]
pub struct ForecastResponse {
    pub period: String,
    pub forecast: ForecastValues,
    pub breakdown_by_stage: serde_json::Value,
}

#[derive(Debug, Serialize)]
pub struct ForecastValues {
    pub best_case: f64,
    pub most_likely: f64,
    pub worst_case: f64,
    pub weighted: f64,
}

// ============================================================================
// CRM - ACCOUNTS
// ============================================================================

#[derive(Debug, Serialize, Deserialize, FromRow, Clone)]
pub struct Account {
    pub id: Uuid,
    pub tenant_id: Uuid,
    pub name: String,
    pub cnpj: Option<String>,
    pub industry: Option<String>,
    pub employees_count: Option<i32>,
    pub annual_revenue: Option<Decimal>,
    pub website: Option<String>,
    #[sqlx(json)]
    pub address: Option<serde_json::Value>,
    pub account_manager_id: Option<Uuid>,
    pub health_score: Option<i32>,
    pub status: String,
    #[sqlx(json)]
    pub custom_fields: serde_json::Value,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Serialize)]
pub struct AccountHealthScore {
    pub account_id: Uuid,
    pub health_score: i32,
    pub status: String,
    pub factors: HealthFactors,
    pub risk_indicators: Vec<RiskIndicator>,
    pub recommendations: Vec<String>,
}

#[derive(Debug, Serialize)]
pub struct HealthFactors {
    pub engagement: FactorScore,
    pub financial: FactorScore,
    pub support_tickets: FactorScore,
    pub product_usage: FactorScore,
}

#[derive(Debug, Serialize)]
pub struct FactorScore {
    pub score: i32,
    pub weight: i32,
    pub trend: String,
}

#[derive(Debug, Serialize)]
pub struct RiskIndicator {
    pub r#type: String,
    pub severity: String,
    pub message: String,
}

// ============================================================================
// CRM - CONTACTS
// ============================================================================

#[derive(Debug, Serialize, Deserialize, FromRow)]
pub struct Contact {
    pub id: Uuid,
    pub tenant_id: Uuid,
    pub account_id: Option<Uuid>,
    pub name: String,
    pub email: Option<String>,
    pub phone: Option<String>,
    pub position: Option<String>,
    pub department: Option<String>,
    pub is_decision_maker: bool,
    pub linkedin_url: Option<String>,
    #[sqlx(json)]
    pub custom_fields: serde_json::Value,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Deserialize, Validate)]
pub struct CreateContactRequest {
    pub account_id: Option<Uuid>,

    #[validate(length(min = 2, max = 255))]
    pub name: String,

    #[validate(email)]
    pub email: Option<String>,

    #[validate(length(max = 20))]
    pub phone: Option<String>,

    #[validate(length(max = 100))]
    pub position: Option<String>,

    #[validate(length(max = 100))]
    pub department: Option<String>,

    #[serde(default)]
    pub is_decision_maker: bool,

    #[validate(url)]
    pub linkedin_url: Option<String>,

    #[serde(default = "default_json_object")]
    pub custom_fields: serde_json::Value,
}

#[derive(Debug, Deserialize, Validate, Default)]
pub struct UpdateContactRequest {
    pub account_id: Option<Uuid>,

    #[serde(default)]
    #[validate(length(min = 2, max = 255))]
    pub name: Option<String>,

    #[serde(default)]
    #[validate(email)]
    pub email: Option<String>,

    #[serde(default)]
    #[validate(length(max = 20))]
    pub phone: Option<String>,

    #[serde(default)]
    #[validate(length(max = 100))]
    pub position: Option<String>,

    #[serde(default)]
    #[validate(length(max = 100))]
    pub department: Option<String>,

    pub is_decision_maker: Option<bool>,

    #[serde(default)]
    #[validate(url)]
    pub linkedin_url: Option<String>,

    #[serde(default)]
    pub custom_fields: Option<serde_json::Value>,

    #[serde(default)]
    pub clear_email: bool,

    #[serde(default)]
    pub clear_phone: bool,

    #[serde(default)]
    pub clear_position: bool,

    #[serde(default)]
    pub clear_department: bool,

    #[serde(default)]
    pub clear_linkedin: bool,

    #[serde(default)]
    pub clear_account: bool,
}

#[derive(Debug, Deserialize, Clone, Default)]
pub struct ListContactsQuery {
    pub account_id: Option<Uuid>,
    pub is_decision_maker: Option<bool>,
    pub department: Option<String>,
    pub search: Option<String>,
    pub page: Option<i32>,
    pub limit: Option<i32>,
}

// ============================================================================
// CRM - ACTIVITIES
// ============================================================================

#[derive(Debug, Clone, Serialize, Deserialize, Type)]
#[sqlx(type_name = "activity_type", rename_all = "lowercase")]
pub enum ActivityType {
    Task,
    Call,
    Meeting,
    Email,
    Note,
}

#[derive(Debug, Clone, Serialize, Deserialize, Type)]
#[sqlx(type_name = "activity_status", rename_all = "lowercase")]
pub enum ActivityStatus {
    Scheduled,
    Completed,
    Canceled,
    Overdue,
}

#[derive(Debug, Serialize, Deserialize, FromRow, Clone)]
pub struct Activity {
    pub id: Uuid,
    pub tenant_id: Uuid,
    pub r#type: ActivityType,
    pub subject: String,
    pub description: Option<String>,
    pub status: ActivityStatus,
    pub scheduled_at: Option<DateTime<Utc>>,
    pub completed_at: Option<DateTime<Utc>>,
    pub duration_minutes: Option<i32>,
    pub lead_id: Option<Uuid>,
    pub account_id: Option<Uuid>,
    pub contact_id: Option<Uuid>,
    pub owner_id: Option<Uuid>,
    pub attendees: Vec<Uuid>,
    #[sqlx(json)]
    pub custom_fields: serde_json::Value,
    pub created_at: DateTime<Utc>,
}

fn default_json_object() -> serde_json::Value {
    serde_json::Value::Object(serde_json::Map::new())
}

#[derive(Debug, Deserialize, Validate)]
pub struct CreateActivityRequest {
    pub r#type: ActivityType,

    #[validate(length(min = 3, max = 255))]
    pub subject: String,

    #[validate(length(max = 5000))]
    pub description: Option<String>,

    pub status: Option<ActivityStatus>,
    pub scheduled_at: Option<DateTime<Utc>>,

    #[validate(range(min = 1, max = 1440))]
    pub duration_minutes: Option<i32>,

    pub lead_id: Option<Uuid>,
    pub account_id: Option<Uuid>,
    pub contact_id: Option<Uuid>,
    pub owner_id: Option<Uuid>,

    #[serde(default)]
    pub attendees: Vec<Uuid>,

    #[serde(default = "default_json_object")]
    pub custom_fields: serde_json::Value,
}

#[derive(Debug, Deserialize, Validate, Default)]
pub struct UpdateActivityRequest {
    #[serde(default)]
    #[validate(length(min = 3, max = 255))]
    pub subject: Option<String>,

    #[serde(default)]
    #[validate(length(max = 5000))]
    pub description: Option<String>,

    pub status: Option<ActivityStatus>,
    pub scheduled_at: Option<DateTime<Utc>>,
    pub completed_at: Option<DateTime<Utc>>,

    #[validate(range(min = 1, max = 1440))]
    pub duration_minutes: Option<i32>,

    pub lead_id: Option<Uuid>,
    pub account_id: Option<Uuid>,
    pub contact_id: Option<Uuid>,
    pub owner_id: Option<Uuid>,
    pub r#type: Option<ActivityType>,

    #[serde(default)]
    pub attendees: Option<Vec<Uuid>>,

    #[serde(default)]
    pub custom_fields: Option<serde_json::Value>,
}

#[cfg(test)]
mod activity_tests {
    use super::*;
    use serde_json::json;
    use validator::Validate;

    #[test]
    fn test_create_activity_request_validation() {
        let mut request = CreateActivityRequest {
            r#type: ActivityType::Call,
            subject: "Follow-up call".to_string(),
            description: None,
            status: None,
            scheduled_at: None,
            duration_minutes: Some(30),
            lead_id: Some(Uuid::new_v4()),
            account_id: None,
            contact_id: None,
            owner_id: None,
            attendees: vec![],
            custom_fields: json!({}),
        };

        assert!(request.validate().is_ok());

        request.subject = "ok".to_string();
        assert!(request.validate().is_ok());

        request.subject = "a".to_string();
        assert!(request.validate().is_err());

        request.subject = "Valid subject".to_string();
        request.duration_minutes = Some(0);
        assert!(request.validate().is_err());
    }

    #[test]
    fn test_default_json_object() {
        let value = default_json_object();
        assert!(value.is_object());
        assert!(value.as_object().unwrap().is_empty());
    }
}

#[derive(Debug, Deserialize, Clone)]
pub struct ListActivitiesQuery {
    pub status: Option<ActivityStatus>,
    pub r#type: Option<ActivityType>,
    pub owner_id: Option<Uuid>,
    pub lead_id: Option<Uuid>,
    pub account_id: Option<Uuid>,
    pub contact_id: Option<Uuid>,
    pub from: Option<DateTime<Utc>>,
    pub to: Option<DateTime<Utc>>,
    pub page: Option<i32>,
    pub limit: Option<i32>,
}

// ============================================================================
// FINANCEIRO
// ============================================================================

#[derive(Debug, Clone, Serialize, Deserialize, Type)]
#[sqlx(type_name = "payment_status", rename_all = "lowercase")]
pub enum PaymentStatus {
    Pending,
    Approved,
    Paid,
    Canceled,
    Overdue,
}

#[derive(Debug, Clone, Serialize, Deserialize, Type)]
#[sqlx(type_name = "payment_method", rename_all = "lowercase")]
pub enum PaymentMethod {
    Pix,
    BankTransfer,
    BankSlip,
    CreditCard,
    Cash,
    Other,
}

#[derive(Debug, Serialize, Deserialize, FromRow)]
pub struct AccountsPayable {
    pub id: Uuid,
    pub tenant_id: Uuid,
    pub supplier_id: Option<Uuid>,
    pub invoice_number: Option<String>,
    pub description: String,
    pub amount: Decimal,
    pub due_date: NaiveDate,
    pub payment_date: Option<NaiveDate>,
    pub payment_method: Option<PaymentMethod>,
    pub status: PaymentStatus,
    pub category: Option<String>,
    pub cost_center: Option<String>,
    pub notes: Option<String>,
    pub attachment_url: Option<String>,
    pub created_by: Option<Uuid>,
    pub approved_by: Option<Uuid>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Deserialize, Validate)]
pub struct CreateAccountsPayableRequest {
    #[validate(length(min = 3, max = 500))]
    pub description: String,

    pub amount: f64,
    pub due_date: NaiveDate,

    #[validate(length(max = 100))]
    pub invoice_number: Option<String>,

    pub supplier_id: Option<Uuid>,
    pub category: Option<String>,
    pub cost_center: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, FromRow)]
pub struct AccountsReceivable {
    pub id: Uuid,
    pub tenant_id: Uuid,
    pub customer_id: Option<Uuid>,
    pub invoice_number: Option<String>,
    pub description: String,
    pub amount: Decimal,
    pub due_date: NaiveDate,
    pub payment_date: Option<NaiveDate>,
    pub payment_method: Option<PaymentMethod>,
    pub status: PaymentStatus,
    pub notes: Option<String>,
    pub nfe_key: Option<String>,
    pub nfe_url: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CashflowProjection {
    pub month: String,
    pub opening_balance: f64,
    pub inflows: Inflows,
    pub outflows: Outflows,
    pub closing_balance: f64,
    pub ai_confidence: f32,
}

#[derive(Debug, Serialize)]
pub struct Inflows {
    pub sales: f64,
    pub receivables: f64,
    pub other: f64,
}

#[derive(Debug, Serialize)]
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

// ============================================================================
// RECURSOS HUMANOS
// ============================================================================

#[derive(Debug, Clone, Serialize, Deserialize, Type)]
#[sqlx(type_name = "employment_type", rename_all = "lowercase")]
pub enum EmploymentType {
    CLT,
    PJ,
    Intern,
    Contractor,
}

#[derive(Debug, Clone, Serialize, Deserialize, Type)]
#[sqlx(type_name = "employee_status", rename_all = "lowercase")]
pub enum EmployeeStatus {
    Active,
    OnLeave,
    Resigned,
    Terminated,
}

#[derive(Debug, Serialize, Deserialize, FromRow)]
pub struct Employee {
    pub id: Uuid,
    pub tenant_id: Uuid,
    pub user_id: Option<Uuid>,
    pub full_name: String,
    pub cpf: String,
    pub rg: Option<String>,
    pub birth_date: Option<NaiveDate>,
    pub email: Option<String>,
    pub phone: Option<String>,
    #[sqlx(json)]
    pub address: Option<serde_json::Value>,
    pub employment_type: EmploymentType,
    pub status: EmployeeStatus,
    pub department: Option<String>,
    pub position: Option<String>,
    pub manager_id: Option<Uuid>,
    pub admission_date: NaiveDate,
    pub resignation_date: Option<NaiveDate>,
    pub base_salary: Option<Decimal>,
    #[sqlx(json)]
    pub benefits: serde_json::Value,
    #[sqlx(json)]
    pub bank_info: Option<serde_json::Value>,
    pub performance_score: Option<Decimal>,
    #[sqlx(json)]
    pub custom_fields: serde_json::Value,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct PayrollCalculation {
    pub employee_id: Uuid,
    pub name: String,
    pub gross_salary: f64,
    pub overtime: f64,
    pub benefits: f64,
    pub deductions: PayrollDeductions,
    pub net_salary: f64,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct PayrollDeductions {
    pub inss: f64,
    pub irrf: f64,
    pub health_plan: f64,
    pub other: f64,
}

// ============================================================================
// USER & TENANT
// ============================================================================

#[derive(Debug, Serialize, Deserialize, FromRow, Clone)]
pub struct User {
    pub id: Uuid,
    pub tenant_id: Uuid,
    pub email: String,
    pub password_hash: Option<String>,
    pub name: String,
    pub avatar_url: Option<String>,
    pub phone: Option<String>,
    pub language: String,
    pub timezone: String,
    pub status: String,
    pub last_login_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Serialize, Deserialize, FromRow)]
pub struct Tenant {
    pub id: Uuid,
    pub name: String,
    pub domain: String,
    pub plan: String,
    pub status: String,
    #[sqlx(json)]
    pub settings: serde_json::Value,
    pub max_users: i32,
    pub storage_limit_gb: i32,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Serialize, Clone)]
pub struct UserSummary {
    pub id: Uuid,
    pub name: String,
    pub email: String,
}

// ============================================================================
// PAGINAÇÃO
// ============================================================================

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

impl<T> PaginatedResponse<T> {
    pub fn new(data: Vec<T>, page: i32, limit: i32, total_items: i64) -> Self {
        let total_pages = (total_items as f64 / limit as f64).ceil() as i32;

        Self {
            data,
            pagination: Pagination {
                current_page: page,
                total_pages,
                total_items,
                per_page: limit,
            },
        }
    }
}

// ============================================================================
// ANALYTICS
// ============================================================================

#[derive(Debug, Serialize)]
pub struct KPIResponse {
    pub period: String,
    pub mrr: KPIMetric,
    pub arr: KPIMetric,
    pub cac: KPIMetric,
    pub ltv: KPIMetric,
    pub ltv_cac_ratio: f64,
    pub churn_rate: KPIMetric,
    pub burn_rate: BurnRate,
}

#[derive(Debug, Serialize)]
pub struct KPIMetric {
    pub value: f64,
    pub change: f64,
    pub trend: String,
}

#[derive(Debug, Serialize)]
pub struct BurnRate {
    pub value: f64,
    pub runway_months: f64,
}
