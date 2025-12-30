//! Arcsat Market Intelligence Module
//!
//! Sistema de scraping e análise de marketplaces integrado ao ERP
//!
//! Features:
//! - Web scraping resiliente com headless Chrome
//! - Suporte a múltiplos marketplaces (Amazon, Mercado Livre, B2W, etc)
//! - Sistema de filas com Redis
//! - Análise de tendências e competidores
//! - Alertas de oportunidades de mercado

pub mod scrapers;
pub mod analysis;
pub mod queue;
pub mod proxy;
pub mod models;
pub mod api;

pub use models::*;
pub use api::router;

use arcsat_core::Result;

/// Engine principal de scraping
pub struct MarketIntelligenceEngine {
    pub scrapers: scrapers::ScraperRegistry,
    pub queue: queue::JobQueue,
    pub analysis: analysis::TrendAnalyzer,
}

impl MarketIntelligenceEngine {
    pub async fn new(redis_url: &str, proxy_config: Option<proxy::ProxyConfig>) -> Result<Self> {
        Ok(Self {
            scrapers: scrapers::ScraperRegistry::new(proxy_config),
            queue: queue::JobQueue::new(redis_url).await?,
            analysis: analysis::TrendAnalyzer::new(),
        })
    }

    pub async fn submit_job(&self, job: models::ScrapingJob) -> Result<String> {
        self.queue.enqueue(job).await
    }

    pub async fn get_job_status(&self, job_id: &str) -> Result<models::JobStatus> {
        self.queue.get_status(job_id).await
    }
}
