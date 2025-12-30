use serde::{Deserialize, Serialize};
use uuid::Uuid;
use chrono::{DateTime, Utc};

/// Marketplace suportado
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum Marketplace {
    Amazon,
    MercadoLivre,
    B2W,
    Magalu,
    Shopee,
    AliExpress,
}

impl Marketplace {
    pub fn base_url(&self) -> &'static str {
        match self {
            Marketplace::Amazon => "https://www.amazon.com.br",
            Marketplace::MercadoLivre => "https://www.mercadolivre.com.br",
            Marketplace::B2W => "https://www.americanas.com.br",
            Marketplace::Magalu => "https://www.magazineluiza.com.br",
            Marketplace::Shopee => "https://shopee.com.br",
            Marketplace::AliExpress => "https://pt.aliexpress.com",
        }
    }
}

/// Status do job de scraping
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum JobStatus {
    Pending,
    Running,
    Completed,
    Failed,
    Cancelled,
}

/// Job de scraping
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScrapingJob {
    pub id: Uuid,
    pub tenant_id: Uuid,
    pub marketplace: Marketplace,
    pub search_query: String,
    pub category: Option<String>,
    pub max_pages: u32,
    pub priority: u8, // 1-10
    pub status: JobStatus,
    pub created_at: DateTime<Utc>,
    pub started_at: Option<DateTime<Utc>>,
    pub completed_at: Option<DateTime<Utc>>,
    pub error: Option<String>,
}

impl ScrapingJob {
    pub fn new(
        tenant_id: Uuid,
        marketplace: Marketplace,
        search_query: String,
        max_pages: u32,
    ) -> Self {
        Self {
            id: Uuid::new_v4(),
            tenant_id,
            marketplace,
            search_query,
            category: None,
            max_pages,
            priority: 5,
            status: JobStatus::Pending,
            created_at: Utc::now(),
            started_at: None,
            completed_at: None,
            error: None,
        }
    }
}

/// Produto encontrado no scraping
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScrapedProduct {
    pub id: Uuid,
    pub job_id: Uuid,
    pub marketplace: Marketplace,

    // Dados do produto
    pub external_id: String, // ID no marketplace
    pub title: String,
    pub price: f64,
    pub currency: String,
    pub url: String,
    pub image_url: Option<String>,

    // Vendedor
    pub seller_name: String,
    pub seller_id: Option<String>,
    pub seller_rating: Option<f64>,

    // Métricas
    pub sales_rank: Option<i32>,
    pub rating: Option<f64>,
    pub num_reviews: i32,
    pub availability: bool,

    // Metadata
    pub category: Option<String>,
    pub brand: Option<String>,
    pub scraped_at: DateTime<Utc>,

    // Dados adicionais (JSON flexível)
    pub extra: serde_json::Value,
}

/// Análise de tendência
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TrendAnalysis {
    pub id: Uuid,
    pub tenant_id: Uuid,
    pub marketplace: Marketplace,
    pub category: String,
    pub period_start: DateTime<Utc>,
    pub period_end: DateTime<Utc>,

    // Estatísticas
    pub total_products: u64,
    pub avg_price: f64,
    pub median_price: f64,
    pub min_price: f64,
    pub max_price: f64,

    // Insights
    pub top_sellers: Vec<String>,
    pub trending_keywords: Vec<String>,
    pub growth_rate: f64, // Percentual
    pub competition_level: CompetitionLevel,

    pub analyzed_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum CompetitionLevel {
    Low,
    Medium,
    High,
    VeryHigh,
}

/// Request para criar job
#[derive(Debug, Clone, Deserialize)]
pub struct CreateJobRequest {
    pub marketplace: Marketplace,
    pub search_query: String,
    pub category: Option<String>,
    pub max_pages: u32,
    pub priority: Option<u8>,
}

/// Response com job criado
#[derive(Debug, Clone, Serialize)]
pub struct CreateJobResponse {
    pub job_id: Uuid,
    pub status: JobStatus,
    pub message: String,
}
