use crate::models::*;
use std::collections::HashMap;

/// Analisador de tendências de mercado
pub struct TrendAnalyzer {
    // Cache de análises
    cache: HashMap<String, TrendAnalysis>,
}

impl TrendAnalyzer {
    pub fn new() -> Self {
        Self {
            cache: HashMap::new(),
        }
    }

    /// Analisa produtos coletados e gera insights
    pub fn analyze(
        &mut self,
        tenant_id: uuid::Uuid,
        marketplace: Marketplace,
        category: &str,
        products: &[ScrapedProduct],
    ) -> TrendAnalysis {
        let mut prices: Vec<f64> = products.iter().map(|p| p.price).collect();
        prices.sort_by(|a, b| a.partial_cmp(b).unwrap());

        let total_products = products.len() as u64;
        let avg_price = if !prices.is_empty() {
            prices.iter().sum::<f64>() / prices.len() as f64
        } else {
            0.0
        };

        let median_price = if !prices.is_empty() {
            prices[prices.len() / 2]
        } else {
            0.0
        };

        let min_price = prices.first().copied().unwrap_or(0.0);
        let max_price = prices.last().copied().unwrap_or(0.0);

        // Top sellers
        let mut seller_counts: HashMap<String, usize> = HashMap::new();
        for product in products {
            *seller_counts.entry(product.seller_name.clone()).or_insert(0) += 1;
        }

        let mut top_sellers: Vec<(String, usize)> = seller_counts.into_iter().collect();
        top_sellers.sort_by(|a, b| b.1.cmp(&a.1));
        let top_sellers: Vec<String> = top_sellers
            .into_iter()
            .take(10)
            .map(|(name, _)| name)
            .collect();

        // Keywords mais frequentes nos títulos
        let trending_keywords = self.extract_keywords(products);

        // Nível de competição baseado no número de vendedores únicos
        let unique_sellers = seller_counts.len();
        let competition_level = match unique_sellers {
            0..=10 => CompetitionLevel::Low,
            11..=50 => CompetitionLevel::Medium,
            51..=100 => CompetitionLevel::High,
            _ => CompetitionLevel::VeryHigh,
        };

        let analysis = TrendAnalysis {
            id: uuid::Uuid::new_v4(),
            tenant_id,
            marketplace,
            category: category.to_string(),
            period_start: chrono::Utc::now() - chrono::Duration::days(7),
            period_end: chrono::Utc::now(),
            total_products,
            avg_price,
            median_price,
            min_price,
            max_price,
            top_sellers,
            trending_keywords,
            growth_rate: 0.0, // TODO: Calcular com base em histórico
            competition_level,
            analyzed_at: chrono::Utc::now(),
        };

        // Cache
        let cache_key = format!("{}:{:?}:{}", tenant_id, marketplace, category);
        self.cache.insert(cache_key, analysis.clone());

        analysis
    }

    /// Extrai keywords mais frequentes dos títulos
    fn extract_keywords(&self, products: &[ScrapedProduct]) -> Vec<String> {
        let mut word_counts: HashMap<String, usize> = HashMap::new();

        // Stop words para filtrar
        let stop_words: std::collections::HashSet<&str> = [
            "de", "da", "do", "para", "com", "em", "por", "e", "a", "o",
            "the", "and", "or", "for", "with", "in", "on", "at"
        ].iter().copied().collect();

        for product in products {
            let words: Vec<&str> = product.title
                .to_lowercase()
                .split(|c: char| !c.is_alphanumeric())
                .filter(|w| w.len() > 3 && !stop_words.contains(w))
                .collect();

            for word in words {
                *word_counts.entry(word.to_string()).or_insert(0) += 1;
            }
        }

        let mut keywords: Vec<(String, usize)> = word_counts.into_iter().collect();
        keywords.sort_by(|a, b| b.1.cmp(&a.1));

        keywords
            .into_iter()
            .take(20)
            .map(|(word, _)| word)
            .collect()
    }
}

impl Default for TrendAnalyzer {
    fn default() -> Self {
        Self::new()
    }
}
