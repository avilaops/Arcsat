use arcsat_core::Result;
use crate::models::*;
use crate::proxy::ProxyConfig;
use headless_chrome::{Browser, LaunchOptions, Tab};
use scraper::{Html, Selector};
use std::sync::Arc;
use std::time::Duration;
use tracing::{info, warn, error};

/// Registry de scrapers por marketplace
pub struct ScraperRegistry {
    proxy_config: Option<ProxyConfig>,
}

impl ScraperRegistry {
    pub fn new(proxy_config: Option<ProxyConfig>) -> Self {
        Self { proxy_config }
    }

    pub async fn scrape(&self, job: &ScrapingJob) -> Result<Vec<ScrapedProduct>> {
        info!("Starting scraping job {} for {:?}", job.id, job.marketplace);

        match job.marketplace {
            Marketplace::Amazon => self.scrape_amazon(job).await,
            Marketplace::MercadoLivre => self.scrape_mercado_livre(job).await,
            Marketplace::B2W => self.scrape_b2w(job).await,
            Marketplace::Magalu => self.scrape_magalu(job).await,
            Marketplace::Shopee => self.scrape_shopee(job).await,
            Marketplace::AliExpress => self.scrape_aliexpress(job).await,
        }
    }

    /// Scraper para Amazon BR
    async fn scrape_amazon(&self, job: &ScrapingJob) -> Result<Vec<ScrapedProduct>> {
        let browser = self.create_browser()?;
        let tab = browser.new_tab()?;

        let mut products = Vec::new();
        let base_url = "https://www.amazon.com.br";
        let search_url = format!("{}/s?k={}", base_url, urlencoding::encode(&job.search_query));

        for page in 1..=job.max_pages {
            info!("Scraping Amazon page {} of {}", page, job.max_pages);

            let url = if page == 1 {
                search_url.clone()
            } else {
                format!("{}&page={}", search_url, page)
            };

            tab.navigate_to(&url)?;
            tab.wait_for_element("div[data-component-type='s-search-result']")?;

            // Delay aleatório humanizado
            std::thread::sleep(Duration::from_millis(2000 + (rand::random::<u64>() % 3000)));

            let html = tab.get_content()?;
            let document = Html::parse_document(&html);

            // Seletores Amazon
            let product_selector = Selector::parse("div[data-component-type='s-search-result']").unwrap();
            let title_selector = Selector::parse("h2 a span").unwrap();
            let price_selector = Selector::parse("span.a-price-whole").unwrap();
            let rating_selector = Selector::parse("span.a-icon-alt").unwrap();
            let link_selector = Selector::parse("h2 a").unwrap();

            for element in document.select(&product_selector) {
                let title = element
                    .select(&title_selector)
                    .next()
                    .map(|e| e.text().collect::<String>())
                    .unwrap_or_default();

                let price_text = element
                    .select(&price_selector)
                    .next()
                    .map(|e| e.text().collect::<String>())
                    .unwrap_or_else(|| "0".to_string());

                let price = price_text
                    .replace(".", "")
                    .replace(",", ".")
                    .parse::<f64>()
                    .unwrap_or(0.0);

                let rating_text = element
                    .select(&rating_selector)
                    .next()
                    .map(|e| e.text().collect::<String>())
                    .unwrap_or_else(|| "0".to_string());

                let rating = rating_text
                    .split_whitespace()
                    .next()
                    .and_then(|s| s.replace(",", ".").parse::<f64>().ok());

                let url = element
                    .select(&link_selector)
                    .next()
                    .and_then(|e| e.value().attr("href"))
                    .map(|href| {
                        if href.starts_with("http") {
                            href.to_string()
                        } else {
                            format!("{}{}", base_url, href)
                        }
                    })
                    .unwrap_or_default();

                let external_id = url
                    .split('/')
                    .find(|s| s.starts_with("dp") || s.len() == 10)
                    .unwrap_or("")
                    .to_string();

                if !title.is_empty() && price > 0.0 {
                    products.push(ScrapedProduct {
                        id: uuid::Uuid::new_v4(),
                        job_id: job.id,
                        marketplace: Marketplace::Amazon,
                        external_id,
                        title,
                        price,
                        currency: "BRL".to_string(),
                        url,
                        image_url: None,
                        seller_name: "Amazon".to_string(),
                        seller_id: None,
                        seller_rating: None,
                        sales_rank: None,
                        rating,
                        num_reviews: 0,
                        availability: true,
                        category: job.category.clone(),
                        brand: None,
                        scraped_at: chrono::Utc::now(),
                        extra: serde_json::json!({}),
                    });
                }
            }

            // Check se tem próxima página
            if page >= job.max_pages {
                break;
            }
        }

        info!("Scraped {} products from Amazon", products.len());
        Ok(products)
    }

    /// Scraper para Mercado Livre
    async fn scrape_mercado_livre(&self, job: &ScrapingJob) -> Result<Vec<ScrapedProduct>> {
        let browser = self.create_browser()?;
        let tab = browser.new_tab()?;

        let mut products = Vec::new();
        let base_url = "https://lista.mercadolivre.com.br";
        let search_slug = job.search_query.replace(" ", "-");

        for page in 1..=job.max_pages {
            info!("Scraping Mercado Livre page {} of {}", page, job.max_pages);

            let offset = (page - 1) * 50;
            let url = format!("{}/{}/_Desde_{}", base_url, search_slug, offset);

            tab.navigate_to(&url)?;
            std::thread::sleep(Duration::from_millis(2000 + (rand::random::<u64>() % 3000)));

            let html = tab.get_content()?;
            let document = Html::parse_document(&html);

            let product_selector = Selector::parse("li.ui-search-layout__item").unwrap();
            let title_selector = Selector::parse("h2.ui-search-item__title").unwrap();
            let price_selector = Selector::parse("span.andes-money-amount__fraction").unwrap();
            let link_selector = Selector::parse("a.ui-search-link").unwrap();

            for element in document.select(&product_selector) {
                let title = element
                    .select(&title_selector)
                    .next()
                    .map(|e| e.text().collect::<String>())
                    .unwrap_or_default();

                let price_text = element
                    .select(&price_selector)
                    .next()
                    .map(|e| e.text().collect::<String>())
                    .unwrap_or_else(|| "0".to_string());

                let price = price_text
                    .replace(".", "")
                    .replace(",", ".")
                    .parse::<f64>()
                    .unwrap_or(0.0);

                let url = element
                    .select(&link_selector)
                    .next()
                    .and_then(|e| e.value().attr("href"))
                    .unwrap_or("")
                    .to_string();

                if !title.is_empty() && price > 0.0 {
                    products.push(ScrapedProduct {
                        id: uuid::Uuid::new_v4(),
                        job_id: job.id,
                        marketplace: Marketplace::MercadoLivre,
                        external_id: url.split('/').last().unwrap_or("").to_string(),
                        title,
                        price,
                        currency: "BRL".to_string(),
                        url,
                        image_url: None,
                        seller_name: "Mercado Livre".to_string(),
                        seller_id: None,
                        seller_rating: None,
                        sales_rank: None,
                        rating: None,
                        num_reviews: 0,
                        availability: true,
                        category: job.category.clone(),
                        brand: None,
                        scraped_at: chrono::Utc::now(),
                        extra: serde_json::json!({}),
                    });
                }
            }
        }

        info!("Scraped {} products from Mercado Livre", products.len());
        Ok(products)
    }

    // Stubs para outros marketplaces
    async fn scrape_b2w(&self, job: &ScrapingJob) -> Result<Vec<ScrapedProduct>> {
        warn!("B2W scraper not implemented yet");
        Ok(Vec::new())
    }

    async fn scrape_magalu(&self, job: &ScrapingJob) -> Result<Vec<ScrapedProduct>> {
        warn!("Magalu scraper not implemented yet");
        Ok(Vec::new())
    }

    async fn scrape_shopee(&self, job: &ScrapingJob) -> Result<Vec<ScrapedProduct>> {
        warn!("Shopee scraper not implemented yet");
        Ok(Vec::new())
    }

    async fn scrape_aliexpress(&self, job: &ScrapingJob) -> Result<Vec<ScrapedProduct>> {
        warn!("AliExpress scraper not implemented yet");
        Ok(Vec::new())
    }

    /// Cria instância do navegador com configurações stealth
    fn create_browser(&self) -> Result<Browser> {
        let mut launch_options = LaunchOptions::default_builder()
            .headless(true)
            .window_size(Some((1920, 1080)))
            .build()
            .map_err(|e| arcsat_core::ArcsatError::Scraping(e.to_string()))?;

        // Adicionar proxy se configurado
        if let Some(proxy) = &self.proxy_config {
            launch_options.proxy_server = Some(proxy.url.clone());
        }

        Browser::new(launch_options)
            .map_err(|e| arcsat_core::ArcsatError::Scraping(e.to_string()))
    }
}
