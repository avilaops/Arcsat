use serde::{Deserialize, Serialize};
use uuid::Uuid;
use chrono::{DateTime, Utc};

/// Lead no CRM
#[derive(Debug, Clone, Serialize, Deserialize)]
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
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum LeadStage {
    New,
    Contacted,
    Qualification,
    Proposal,
    Negotiation,
    Won,
    Lost,
}

/// Produto no catálogo
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Product {
    pub id: Uuid,
    pub tenant_id: Uuid,
    pub sku: String,
    pub name: String,
    pub description: Option<String>,
    pub category: String,
    pub price: f64,
    pub cost: f64,
    pub stock: i32,
    pub active: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Oportunidade de venda
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Opportunity {
    pub id: Uuid,
    pub tenant_id: Uuid,
    pub lead_id: Uuid,
    pub product_id: Uuid,
    pub value: f64,
    pub stage: LeadStage,
    pub expected_close_date: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Insight de market intelligence para CRM
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MarketInsight {
    pub id: Uuid,
    pub product_id: Uuid,
    pub insight_type: InsightType,
    pub title: String,
    pub description: String,
    pub suggested_action: String,
    pub priority: InsightPriority,
    pub data: serde_json::Value,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum InsightType {
    PricingOpportunity,
    HighDemand,
    LowCompetition,
    TrendingProduct,
    PriceAlert,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum InsightPriority {
    Low,
    Medium,
    High,
    Critical,
}
