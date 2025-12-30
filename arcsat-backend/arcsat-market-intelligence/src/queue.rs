use arcsat_core::Result;
use crate::models::{ScrapingJob, JobStatus};
use redis::aio::ConnectionManager;
use redis::AsyncCommands;
use tracing::{info, error};

pub struct JobQueue {
    redis: ConnectionManager,
}

impl JobQueue {
    pub async fn new(redis_url: &str) -> Result<Self> {
        let client = redis::Client::open(redis_url)
            .map_err(|e| arcsat_core::ArcsatError::Internal(e.to_string()))?;

        let redis = ConnectionManager::new(client).await
            .map_err(|e| arcsat_core::ArcsatError::Internal(e.to_string()))?;

        Ok(Self { redis })
    }

    pub async fn enqueue(&self, job: ScrapingJob) -> Result<String> {
        let mut conn = self.redis.clone();
        let job_id = job.id.to_string();
        let job_json = serde_json::to_string(&job)?;

        // Salvar job no Redis
        conn.hset(&format!("job:{}", job_id), "data", &job_json).await
            .map_err(|e| arcsat_core::ArcsatError::Internal(e.to_string()))?;

        // Adicionar à fila por prioridade
        let queue_key = format!("queue:priority:{}", job.priority);
        conn.lpush(&queue_key, &job_id).await
            .map_err(|e| arcsat_core::ArcsatError::Internal(e.to_string()))?;

        info!("Job {} enqueued with priority {}", job_id, job.priority);
        Ok(job_id)
    }

    pub async fn dequeue(&self, priority: u8) -> Result<Option<ScrapingJob>> {
        let mut conn = self.redis.clone();
        let queue_key = format!("queue:priority:{}", priority);

        let job_id: Option<String> = conn.rpop(&queue_key, None).await
            .map_err(|e| arcsat_core::ArcsatError::Internal(e.to_string()))?;

        if let Some(job_id) = job_id {
            let job_json: String = conn.hget(&format!("job:{}", job_id), "data").await
                .map_err(|e| arcsat_core::ArcsatError::Internal(e.to_string()))?;

            let job: ScrapingJob = serde_json::from_str(&job_json)?;
            Ok(Some(job))
        } else {
            Ok(None)
        }
    }

    pub async fn update_status(&self, job_id: &str, status: JobStatus) -> Result<()> {
        let mut conn = self.redis.clone();

        conn.hset(&format!("job:{}", job_id), "status", format!("{:?}", status)).await
            .map_err(|e| arcsat_core::ArcsatError::Internal(e.to_string()))?;

        Ok(())
    }

    pub async fn get_status(&self, job_id: &str) -> Result<JobStatus> {
        let mut conn = self.redis.clone();

        let status_str: String = conn.hget(&format!("job:{}", job_id), "status").await
            .map_err(|e| arcsat_core::ArcsatError::NotFound(format!("Job {} not found", job_id)))?;

        match status_str.as_str() {
            "Pending" => Ok(JobStatus::Pending),
            "Running" => Ok(JobStatus::Running),
            "Completed" => Ok(JobStatus::Completed),
            "Failed" => Ok(JobStatus::Failed),
            "Cancelled" => Ok(JobStatus::Cancelled),
            _ => Ok(JobStatus::Pending),
        }
    }

    pub async fn save_results(&self, job_id: &str, products: &[crate::models::ScrapedProduct]) -> Result<()> {
        let mut conn = self.redis.clone();
        let results_json = serde_json::to_string(products)?;

        conn.hset(&format!("job:{}", job_id), "results", results_json).await
            .map_err(|e| arcsat_core::ArcsatError::Internal(e.to_string()))?;

        conn.hset(&format!("job:{}", job_id), "results_count", products.len()).await
            .map_err(|e| arcsat_core::ArcsatError::Internal(e.to_string()))?;

        Ok(())
    }
}
