//! Market Intelligence Insights para CRM
//!
//! Gera insights acionáveis baseados em dados de scraping

use crate::models::*;
use arcsat_market_intelligence::models::{ScrapedProduct, TrendAnalysis, CompetitionLevel};
use arcsat_core::Result;
use uuid::Uuid;
use chrono::Utc;

/// Gerador de insights de mercado para CRM
pub struct InsightGenerator;

impl InsightGenerator {
    /// Gera insights de pricing baseado em análise de mercado
    pub fn generate_pricing_insights(
        product: &Product,
        market_data: &[ScrapedProduct],
    ) -> Vec<MarketInsight> {
        let mut insights = Vec::new();

        if market_data.is_empty() {
            return insights;
        }

        // Calcular preço médio do mercado
        let market_avg = market_data.iter()
            .map(|p| p.price)
            .sum::<f64>() / market_data.len() as f64;

        let price_diff_percent = ((product.price - market_avg) / market_avg) * 100.0;

        // Insight: Preço muito acima do mercado
        if price_diff_percent > 20.0 {
            insights.push(MarketInsight {
                id: Uuid::new_v4(),
                product_id: product.id,
                insight_type: InsightType::PricingOpportunity,
                title: "Preço acima do mercado".to_string(),
                description: format!(
                    "Seu produto está {}% mais caro que a média do mercado (R$ {:.2}). \
                    Considere ajustar para aumentar competitividade.",
                    price_diff_percent.abs() as i32,
                    market_avg
                ),
                suggested_action: format!("Reduzir preço para R$ {:.2}", market_avg * 1.05),
                priority: InsightPriority::High,
                data: serde_json::json!({
                    "current_price": product.price,
                    "market_avg": market_avg,
                    "diff_percent": price_diff_percent,
                    "competitors": market_data.len()
                }),
                created_at: Utc::now(),
            });
        }

        // Insight: Preço muito abaixo (oportunidade de aumentar margem)
        if price_diff_percent < -15.0 {
            insights.push(MarketInsight {
                id: Uuid::new_v4(),
                product_id: product.id,
                insight_type: InsightType::PricingOpportunity,
                title: "Oportunidade de aumentar margem".to_string(),
                description: format!(
                    "Seu preço está {}% abaixo da média (R$ {:.2}). \
                    Há espaço para aumentar margem sem perder competitividade.",
                    price_diff_percent.abs() as i32,
                    market_avg
                ),
                suggested_action: format!("Aumentar preço para R$ {:.2}", market_avg * 0.95),
                priority: InsightPriority::Medium,
                data: serde_json::json!({
                    "current_price": product.price,
                    "market_avg": market_avg,
                    "diff_percent": price_diff_percent,
                    "potential_revenue_gain": (market_avg * 0.95 - product.price) * product.stock as f64
                }),
                created_at: Utc::now(),
            });
        }

        // Insight: Preço competitivo (sweet spot)
        if price_diff_percent.abs() <= 10.0 {
            insights.push(MarketInsight {
                id: Uuid::new_v4(),
                product_id: product.id,
                insight_type: InsightType::PricingOpportunity,
                title: "Preço competitivo".to_string(),
                description: format!(
                    "Seu preço está alinhado com o mercado. \
                    Continue monitorando para manter competitividade."
                ),
                suggested_action: "Manter estratégia atual".to_string(),
                priority: InsightPriority::Low,
                data: serde_json::json!({
                    "current_price": product.price,
                    "market_avg": market_avg,
                    "diff_percent": price_diff_percent
                }),
                created_at: Utc::now(),
            });
        }

        insights
    }

    /// Gera insights de demanda baseado em análise de tendências
    pub fn generate_demand_insights(
        product: &Product,
        trend_analysis: &TrendAnalysis,
    ) -> Vec<MarketInsight> {
        let mut insights = Vec::new();

        // Insight: Alta demanda detectada
        if trend_analysis.total_products > 100 {
            insights.push(MarketInsight {
                id: Uuid::new_v4(),
                product_id: product.id,
                insight_type: InsightType::HighDemand,
                title: "Alta demanda detectada".to_string(),
                description: format!(
                    "{} produtos similares no marketplace. Categoria em alta!",
                    trend_analysis.total_products
                ),
                suggested_action: "Aumentar estoque e investir em marketing".to_string(),
                priority: InsightPriority::High,
                data: serde_json::json!({
                    "total_products": trend_analysis.total_products,
                    "trending_keywords": trend_analysis.trending_keywords,
                    "avg_price": trend_analysis.avg_price
                }),
                created_at: Utc::now(),
            });
        }

        // Insight: Baixa competição
        if trend_analysis.competition_level == CompetitionLevel::Low {
            insights.push(MarketInsight {
                id: Uuid::new_v4(),
                product_id: product.id,
                insight_type: InsightType::LowCompetition,
                title: "Baixa competição no nicho".to_string(),
                description: format!(
                    "Poucos vendedores nesta categoria. Oportunidade de dominar o nicho!"
                ),
                suggested_action: "Investir em SEO e anúncios para capturar mercado".to_string(),
                priority: InsightPriority::High,
                data: serde_json::json!({
                    "competition_level": "low",
                    "total_sellers": trend_analysis.top_sellers.len(),
                    "market_size": trend_analysis.total_products
                }),
                created_at: Utc::now(),
            });
        }

        // Insight: Produto trending
        if !trend_analysis.trending_keywords.is_empty() {
            let has_trending = trend_analysis.trending_keywords.iter()
                .any(|kw| product.name.to_lowercase().contains(&kw.to_lowercase()));

            if has_trending {
                insights.push(MarketInsight {
                    id: Uuid::new_v4(),
                    product_id: product.id,
                    insight_type: InsightType::TrendingProduct,
                    title: "Produto com keywords em alta".to_string(),
                    description: format!(
                        "Seu produto contém keywords trending: {}",
                        trend_analysis.trending_keywords[..5.min(trend_analysis.trending_keywords.len())].join(", ")
                    ),
                    suggested_action: "Otimizar título e descrição com essas keywords".to_string(),
                    priority: InsightPriority::Medium,
                    data: serde_json::json!({
                        "trending_keywords": trend_analysis.trending_keywords,
                        "growth_rate": trend_analysis.growth_rate
                    }),
                    created_at: Utc::now(),
                });
            }
        }

        insights
    }

    /// Gera insights de alerta de preço
    pub fn generate_price_alerts(
        product: &Product,
        competitors: &[ScrapedProduct],
    ) -> Vec<MarketInsight> {
        let mut insights = Vec::new();

        // Encontrar competidor mais barato
        if let Some(cheapest) = competitors.iter().min_by(|a, b| {
            a.price.partial_cmp(&b.price).unwrap()
        }) {
            if cheapest.price < product.price * 0.9 {
                insights.push(MarketInsight {
                    id: Uuid::new_v4(),
                    product_id: product.id,
                    insight_type: InsightType::PriceAlert,
                    title: "Competidor com preço muito menor".to_string(),
                    description: format!(
                        "Competidor '{}' está vendendo por R$ {:.2} ({}% mais barato)",
                        cheapest.seller_name,
                        cheapest.price,
                        ((product.price - cheapest.price) / product.price * 100.0) as i32
                    ),
                    suggested_action: format!(
                        "Considere ajustar preço ou agregar valor para justificar diferença"
                    ),
                    priority: InsightPriority::Critical,
                    data: serde_json::json!({
                        "your_price": product.price,
                        "competitor_price": cheapest.price,
                        "competitor": cheapest.seller_name,
                        "marketplace": format!("{:?}", cheapest.marketplace)
                    }),
                    created_at: Utc::now(),
                });
            }
        }

        insights
    }
}

/// Serviço de integração entre Market Intelligence e CRM
pub struct CrmIntegrationService;

impl CrmIntegrationService {
    /// Analisa produto e gera insights completos
    pub async fn analyze_product(
        product: &Product,
        market_data: &[ScrapedProduct],
        trend_analysis: Option<&TrendAnalysis>,
    ) -> Result<Vec<MarketInsight>> {
        let mut all_insights = Vec::new();

        // Insights de pricing
        let pricing_insights = InsightGenerator::generate_pricing_insights(product, market_data);
        all_insights.extend(pricing_insights);

        // Insights de demanda (se tiver análise)
        if let Some(trends) = trend_analysis {
            let demand_insights = InsightGenerator::generate_demand_insights(product, trends);
            all_insights.extend(demand_insights);
        }

        // Alertas de preço
        let price_alerts = InsightGenerator::generate_price_alerts(product, market_data);
        all_insights.extend(price_alerts);

        Ok(all_insights)
    }

    /// Sugere preço ideal baseado em análise de mercado
    pub fn suggest_optimal_price(
        product: &Product,
        market_data: &[ScrapedProduct],
        target_margin: f64, // 0.0 - 1.0 (ex: 0.3 = 30%)
    ) -> f64 {
        if market_data.is_empty() {
            return product.price;
        }

        let market_avg = market_data.iter()
            .map(|p| p.price)
            .sum::<f64>() / market_data.len() as f64;

        // Preço baseado em custo + margem desejada
        let cost_based_price = product.cost * (1.0 + target_margin);

        // Preço baseado em mercado (5% abaixo da média para competitividade)
        let market_based_price = market_avg * 0.95;

        // Escolher o maior entre os dois (garantir margem mínima)
        cost_based_price.max(market_based_price)
    }
}
