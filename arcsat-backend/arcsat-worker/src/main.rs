//! Arcsat Worker - Background Job Processor
//!
//! Worker que processa jobs de scraping em background

use arcsat_market_intelligence::{MarketIntelligenceEngine, models::*};
use std::sync::Arc;
use tokio::time::{sleep, Duration};
use tracing::{info, error, warn};
use signal_hook::consts::signal::*;
use signal_hook_tokio::Signals;
use futures::stream::StreamExt;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Load .env
    dotenv::dotenv().ok();

    // Setup logging
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "arcsat_worker=debug,arcsat_market_intelligence=debug".into())
        )
        .json()
        .init();

    info!("🔄 Starting Arcsat Worker");

    // Config
    let redis_url = std::env::var("REDIS_URL")
        .unwrap_or_else(|_| "redis://localhost:6379".to_string());

    let proxy_enabled = std::env::var("MI_PROXY_ENABLED")
        .unwrap_or_else(|_| "false".to_string())
        .parse::<bool>()
        .unwrap_or(false);

    let proxy_config = if proxy_enabled {
        std::env::var("MI_PROXY_URL").ok().map(|url| {
            arcsat_market_intelligence::proxy::ProxyConfig::new(url)
        })
    } else {
        None
    };

    let max_concurrent = std::env::var("MI_MAX_CONCURRENT_JOBS")
        .unwrap_or_else(|_| "5".to_string())
        .parse::<usize>()
        .unwrap_or(5);

    // Initialize engine
    let engine = Arc::new(
        MarketIntelligenceEngine::new(&redis_url, proxy_config).await?
    );

    info!("✅ Worker initialized (max concurrent: {})", max_concurrent);
    if proxy_enabled {
        info!("🎭 Proxy enabled");
    } else {
        warn!("⚠️  Running WITHOUT proxy - limited to ~10-20 requests");
    }

    // Signal handling
    let signals = Signals::new(&[SIGTERM, SIGINT])?;
    let handle = signals.handle();

    let signals_task = tokio::spawn(handle_signals(signals));

    // Main worker loop
    let worker_handle = tokio::spawn(worker_loop(engine.clone(), max_concurrent));

    // Wait for either signals or worker to finish
    tokio::select! {
        _ = signals_task => {
            info!("📡 Received shutdown signal");
        }
        result = worker_handle => {
            if let Err(e) = result {
                error!("❌ Worker error: {}", e);
            }
        }
    }

    info!("👋 Worker shutting down gracefully");
    handle.close();

    Ok(())
}

async fn handle_signals(mut signals: Signals) {
    while let Some(signal) = signals.next().await {
        match signal {
            SIGTERM | SIGINT => {
                info!("Received signal {}, shutting down", signal);
                break;
            }
            _ => {}
        }
    }
}

async fn worker_loop(engine: Arc<MarketIntelligenceEngine>, max_concurrent: usize) {
    let mut active_tasks = Vec::new();

    loop {
        // Limpar tasks finalizadas
        active_tasks.retain(|task: &tokio::task::JoinHandle<_>| !task.is_finished());

        // Se temos espaço, buscar novos jobs
        if active_tasks.len() < max_concurrent {
            // Tentar buscar job de cada prioridade (10 até 1)
            let mut job_found = false;

            for priority in (1..=10).rev() {
                match engine.queue.dequeue(priority).await {
                    Ok(Some(job)) => {
                        info!("📥 Dequeued job {} (priority {})", job.id, job.priority);

                        let engine_clone = engine.clone();
                        let task = tokio::spawn(async move {
                            process_job(engine_clone, job).await;
                        });

                        active_tasks.push(task);
                        job_found = true;
                        break;
                    }
                    Ok(None) => continue,
                    Err(e) => {
                        error!("❌ Error dequeuing from priority {}: {}", priority, e);
                    }
                }
            }

            if !job_found {
                // Sem jobs, aguardar um pouco
                sleep(Duration::from_secs(2)).await;
            }
        } else {
            // Pool cheio, aguardar
            sleep(Duration::from_millis(500)).await;
        }
    }
}

async fn process_job(engine: Arc<MarketIntelligenceEngine>, mut job: ScrapingJob) {
    info!("🚀 Processing job {}: {} on {:?}",
        job.id, job.search_query, job.marketplace);

    // Update status to running
    job.status = JobStatus::Running;
    job.started_at = Some(chrono::Utc::now());

    if let Err(e) = engine.queue.update_status(&job.id.to_string(), JobStatus::Running).await {
        error!("❌ Failed to update job status: {}", e);
    }

    // Execute scraping
    match engine.scrapers.scrape(&job).await {
        Ok(products) => {
            info!("✅ Job {} completed: {} products found", job.id, products.len());

            // Save results
            if let Err(e) = engine.queue.save_results(&job.id.to_string(), &products).await {
                error!("❌ Failed to save results: {}", e);
            }

            // Update status
            job.status = JobStatus::Completed;
            job.completed_at = Some(chrono::Utc::now());

            if let Err(e) = engine.queue.update_status(&job.id.to_string(), JobStatus::Completed).await {
                error!("❌ Failed to update status: {}", e);
            }

            // Analyze trends
            if !products.is_empty() {
                let analysis = engine.analysis.analyze(
                    job.tenant_id,
                    job.marketplace,
                    job.category.as_deref().unwrap_or("general"),
                    &products
                );

                info!("📊 Analysis complete: avg_price={:.2}, competition={:?}",
                    analysis.avg_price, analysis.competition_level);
            }
        }
        Err(e) => {
            error!("❌ Job {} failed: {}", job.id, e);

            job.status = JobStatus::Failed;
            job.completed_at = Some(chrono::Utc::now());
            job.error = Some(e.to_string());

            if let Err(e) = engine.queue.update_status(&job.id.to_string(), JobStatus::Failed).await {
                error!("❌ Failed to update status: {}", e);
            }
        }
    }

    let duration = job.completed_at.unwrap() - job.started_at.unwrap();
    info!("⏱️  Job {} finished in {} seconds", job.id, duration.num_seconds());
}
