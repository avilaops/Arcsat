// cache.rs - Sistema de Cache com Redis

use redis::{aio::ConnectionManager, AsyncCommands, Client};
use serde::{de::DeserializeOwned, Serialize};
use std::time::Duration;

use crate::error::{AppError, Result};

/// Cache Manager com Redis
#[derive(Clone)]
pub struct CacheManager {
    client: ConnectionManager,
}

impl CacheManager {
    /// Criar novo cache manager
    pub async fn new(redis_url: &str) -> Result<Self> {
        tracing::info!("Connecting to Redis...");

        let client = Client::open(redis_url)
            .map_err(|e| AppError::InternalError(format!("Redis client error: {}", e)))?;

        let connection = ConnectionManager::new(client)
            .await
            .map_err(|e| AppError::InternalError(format!("Redis connection error: {}", e)))?;

        tracing::info!("✅ Connected to Redis");

        Ok(Self { client: connection })
    }

    /// Get value do cache
    pub async fn get<T: DeserializeOwned>(&self, key: &str) -> Result<Option<T>> {
        let mut conn = self.client.clone();

        let value: Option<String> = conn
            .get(key)
            .await
            .map_err(|e| AppError::InternalError(format!("Redis get error: {}", e)))?;

        match value {
            Some(v) => {
                let deserialized = serde_json::from_str(&v)
                    .map_err(|e| AppError::InternalError(format!("JSON deserialize error: {}", e)))?;
                Ok(Some(deserialized))
            }
            None => Ok(None),
        }
    }

    /// Set value no cache com TTL
    pub async fn set<T: Serialize>(
        &self,
        key: &str,
        value: &T,
        ttl: Duration,
    ) -> Result<()> {
        let mut conn = self.client.clone();

        let serialized = serde_json::to_string(value)
            .map_err(|e| AppError::InternalError(format!("JSON serialize error: {}", e)))?;

        let _: () = conn.set_ex(key, serialized, ttl.as_secs())
            .await
            .map_err(|e| AppError::InternalError(format!("Redis set error: {}", e)))?;

        Ok(())
    }

    /// Delete key do cache
    pub async fn delete(&self, key: &str) -> Result<()> {
        let mut conn = self.client.clone();

        let _: () = conn.del(key)
            .await
            .map_err(|e| AppError::InternalError(format!("Redis delete error: {}", e)))?;

        Ok(())
    }

    /// Delete múltiplas keys por pattern
    pub async fn delete_pattern(&self, pattern: &str) -> Result<()> {
        let mut conn = self.client.clone();

        // Buscar keys que fazem match com o pattern
        let keys: Vec<String> = conn
            .keys(pattern)
            .await
            .map_err(|e| AppError::InternalError(format!("Redis keys error: {}", e)))?;

        if !keys.is_empty() {
            let _: () = conn.del(keys)
                .await
                .map_err(|e| AppError::InternalError(format!("Redis delete error: {}", e)))?;
        }

        Ok(())
    }

    /// Check se key existe
    pub async fn exists(&self, key: &str) -> Result<bool> {
        let mut conn = self.client.clone();

        let exists: bool = conn
            .exists(key)
            .await
            .map_err(|e| AppError::InternalError(format!("Redis exists error: {}", e)))?;

        Ok(exists)
    }

    /// Incrementar contador
    pub async fn incr(&self, key: &str) -> Result<i64> {
        let mut conn = self.client.clone();

        let value: i64 = conn
            .incr(key, 1)
            .await
            .map_err(|e| AppError::InternalError(format!("Redis incr error: {}", e)))?;

        Ok(value)
    }

    /// Incrementar com expiry
    pub async fn incr_with_ttl(&self, key: &str, ttl: Duration) -> Result<i64> {
        let mut conn = self.client.clone();

        let value: i64 = conn
            .incr(key, 1)
            .await
            .map_err(|e| AppError::InternalError(format!("Redis incr error: {}", e)))?;

        // Set TTL apenas se for a primeira vez (value == 1)
        if value == 1 {
            let _: () = conn.expire(key, ttl.as_secs() as i64)
                .await
                .map_err(|e| AppError::InternalError(format!("Redis expire error: {}", e)))?;
        }

        Ok(value)
    }

    /// Get ou set (cache-aside pattern)
    pub async fn get_or_set<T, F, Fut>(
        &self,
        key: &str,
        ttl: Duration,
        fetcher: F,
    ) -> Result<T>
    where
        T: Serialize + DeserializeOwned,
        F: FnOnce() -> Fut,
        Fut: std::future::Future<Output = Result<T>>,
    {
        // Tentar buscar do cache
        if let Some(cached) = self.get::<T>(key).await? {
            tracing::debug!("Cache HIT: {}", key);
            return Ok(cached);
        }

        tracing::debug!("Cache MISS: {}", key);

        // Buscar do source
        let value = fetcher().await?;

        // Salvar no cache
        self.set(key, &value, ttl).await?;

        Ok(value)
    }
}

/// Cache keys helper
pub struct CacheKeys;

impl CacheKeys {
    /// Key para usuário
    pub fn user(user_id: &str) -> String {
        format!("user:{}", user_id)
    }

    /// Key para tenant
    pub fn tenant(tenant_id: &str) -> String {
        format!("tenant:{}", tenant_id)
    }

    /// Key para lead
    pub fn lead(tenant_id: &str, lead_id: &str) -> String {
        format!("tenant:{}:lead:{}", tenant_id, lead_id)
    }

    /// Key para lista de leads
    pub fn leads_list(tenant_id: &str, page: i32) -> String {
        format!("tenant:{}:leads:page:{}", tenant_id, page)
    }

    /// Key para forecast
    pub fn forecast(tenant_id: &str) -> String {
        format!("tenant:{}:forecast", tenant_id)
    }

    /// Key para rate limit
    pub fn rate_limit(tenant_id: &str, endpoint: &str) -> String {
        format!("ratelimit:{}:{}", tenant_id, endpoint)
    }

    /// Key para session
    pub fn session(session_id: &str) -> String {
        format!("session:{}", session_id)
    }

    /// Pattern para invalidar cache de um tenant
    pub fn tenant_pattern(tenant_id: &str) -> String {
        format!("tenant:{}:*", tenant_id)
    }
}

/// Cache TTLs
pub struct CacheTTL;

impl CacheTTL {
    /// Usuário: 15 minutos
    pub fn user() -> Duration {
        Duration::from_secs(900)
    }

    /// Tenant: 1 hora
    pub fn tenant() -> Duration {
        Duration::from_secs(3600)
    }

    /// Lead individual: 5 minutos
    pub fn lead() -> Duration {
        Duration::from_secs(300)
    }

    /// Lista de leads: 2 minutos
    pub fn leads_list() -> Duration {
        Duration::from_secs(120)
    }

    /// Forecast: 10 minutos
    pub fn forecast() -> Duration {
        Duration::from_secs(600)
    }

    /// Rate limit: 1 minuto
    pub fn rate_limit() -> Duration {
        Duration::from_secs(60)
    }

    /// Session: 8 horas
    pub fn session() -> Duration {
        Duration::from_secs(28800)
    }
}

/// Cache invalidation helpers
pub struct CacheInvalidation;

impl CacheInvalidation {
    /// Invalidar cache quando lead é modificado
    pub async fn on_lead_modified(
        cache: &CacheManager,
        tenant_id: &str,
        lead_id: &str,
    ) -> Result<()> {
        // Deletar lead específico
        cache.delete(&CacheKeys::lead(tenant_id, lead_id)).await?;

        // Deletar listas de leads (todas as páginas)
        cache.delete_pattern(&format!("tenant:{}:leads:page:*", tenant_id)).await?;

        // Deletar forecast
        cache.delete(&CacheKeys::forecast(tenant_id)).await?;

        tracing::debug!("Cache invalidated for lead {} in tenant {}", lead_id, tenant_id);

        Ok(())
    }

    /// Invalidar cache quando tenant é modificado
    pub async fn on_tenant_modified(
        cache: &CacheManager,
        tenant_id: &str,
    ) -> Result<()> {
        // Deletar tudo relacionado ao tenant
        cache.delete_pattern(&CacheKeys::tenant_pattern(tenant_id)).await?;
        cache.delete(&CacheKeys::tenant(tenant_id)).await?;

        tracing::debug!("Cache invalidated for tenant {}", tenant_id);

        Ok(())
    }
}

/// Exemplo de uso com cache decorator
pub async fn get_lead_cached(
    cache: &CacheManager,
    tenant_id: &str,
    lead_id: &str,
) -> Result<crate::models::Lead> {
    let key = CacheKeys::lead(tenant_id, lead_id);

    cache
        .get_or_set(&key, CacheTTL::lead(), || async {
            // Buscar do banco (simulado)
            // let lead = db.get_lead(tenant_id, lead_id).await?;
            // Ok(lead)

            // Por enquanto, retornar erro
            Err(AppError::not_found("Lead"))
        })
        .await
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    #[ignore] // Requer Redis rodando
    async fn test_cache_set_get() {
        let cache = CacheManager::new("redis://localhost:6379")
            .await
            .expect("Redis connection");

        let key = "test_key";
        let value = "test_value";

        cache
            .set(key, &value, Duration::from_secs(60))
            .await
            .unwrap();

        let cached: Option<String> = cache.get(key).await.unwrap();
        assert_eq!(cached, Some(value.to_string()));
    }

    #[test]
    fn test_cache_keys() {
        assert_eq!(CacheKeys::user("123"), "user:123");
        assert_eq!(CacheKeys::tenant("456"), "tenant:456");
        assert_eq!(
            CacheKeys::lead("tenant-1", "lead-1"),
            "tenant:tenant-1:lead:lead-1"
        );
    }
}
