// middleware/rate_limit.rs - Rate Limiting por Tenant e Plano

use axum::{
    extract::{Request, State},
    http::{HeaderMap, StatusCode},
    middleware::Next,
    response::{IntoResponse, Response},
};
use std::sync::Arc;
use std::time::Duration;

use crate::{
    auth::Claims,
    cache::CacheManager,
    error::AppError,
};

/// Configuração de rate limit por plano
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct RateLimitConfig {
    pub requests_per_minute: u32,
    pub burst_size: u32,
}

impl RateLimitConfig {
    /// Plano Startup: 100 req/min
    pub fn startup() -> Self {
        Self {
            requests_per_minute: 100,
            burst_size: 120,
        }
    }

    /// Plano Business: 500 req/min
    pub fn business() -> Self {
        Self {
            requests_per_minute: 500,
            burst_size: 600,
        }
    }

    /// Plano Enterprise: 2000 req/min
    pub fn enterprise() -> Self {
        Self {
            requests_per_minute: 2000,
            burst_size: 2500,
        }
    }

    /// Admin/Internal: ilimitado (ou muito alto)
    pub fn unlimited() -> Self {
        Self {
            requests_per_minute: 10000,
            burst_size: 15000,
        }
    }

    /// Get config baseado no plano
    pub fn from_plan(plan: &str) -> Self {
        match plan {
            "startup" => Self::startup(),
            "business" => Self::business(),
            "enterprise" => Self::enterprise(),
            _ => Self::startup(),
        }
    }
}

/// Middleware de rate limiting
pub async fn rate_limit_middleware(
    State(cache): State<Arc<CacheManager>>,
    request: Request,
    next: Next,
) -> Result<Response, AppError> {
    // Extrair claims (usuário autenticado)
    let claims = request
        .extensions()
        .get::<Claims>()
        .ok_or_else(|| AppError::Unauthorized("Authentication required".to_string()))?
        .clone();

    let tenant_id = &claims.tenant_id;

    // Buscar configuração do tenant (com cache)
    let config = get_tenant_rate_limit_config(&cache, tenant_id).await?;

    // Verificar rate limit
    let is_allowed = check_rate_limit(&cache, tenant_id, &config).await?;

    if !is_allowed {
        tracing::warn!("Rate limit exceeded for tenant: {}", tenant_id);

        return Err(AppError::RateLimitExceeded);
    }

    // Processar request
    let mut response = next.run(request).await;

    // Adicionar headers de rate limit na response
    add_rate_limit_headers(response.headers_mut(), &cache, tenant_id, &config).await;

    Ok(response)
}

/// Verificar se request está dentro do limite
async fn check_rate_limit(
    cache: &CacheManager,
    tenant_id: &str,
    config: &RateLimitConfig,
) -> Result<bool, AppError> {
    let key = format!("ratelimit:{}:count", tenant_id);
    let ttl = Duration::from_secs(60);

    // Incrementar contador
    let count = cache.incr_with_ttl(&key, ttl).await?;

    // Verificar se excedeu
    Ok(count <= config.burst_size as i64)
}

/// Buscar configuração de rate limit do tenant
async fn get_tenant_rate_limit_config(
    cache: &CacheManager,
    tenant_id: &str,
) -> Result<RateLimitConfig, AppError> {
    // Tentar buscar do cache
    let cache_key = format!("tenant:{}:ratelimit_config", tenant_id);

    if let Some(config) = cache.get::<RateLimitConfig>(&cache_key).await? {
        return Ok(config);
    }

    // Se não tem no cache, buscar do banco (simulado)
    // TODO: Implementar busca real
    let plan = "business"; // Simulado
    let config = RateLimitConfig::from_plan(plan);

    // Salvar no cache (1 hora)
    cache.set(&cache_key, &config, Duration::from_secs(3600)).await?;

    Ok(config)
}

/// Adicionar headers de rate limit na response
async fn add_rate_limit_headers(
    headers: &mut HeaderMap,
    cache: &CacheManager,
    tenant_id: &str,
    config: &RateLimitConfig,
) {
    let key = format!("ratelimit:{}:count", tenant_id);

    if let Ok(Some(count)) = cache.get::<i64>(&key).await {
        let remaining = (config.requests_per_minute as i64 - count).max(0);

        headers.insert("X-RateLimit-Limit", config.requests_per_minute.to_string().parse().unwrap());
        headers.insert("X-RateLimit-Remaining", remaining.to_string().parse().unwrap());
        headers.insert("X-RateLimit-Reset", "60".parse().unwrap()); // 60 segundos
    }
}

/// Middleware específico para endpoints sensíveis (rate limit mais restritivo)
pub fn strict_rate_limit() -> impl Fn(Request, Next) -> std::pin::Pin<Box<dyn std::future::Future<Output = Result<Response, AppError>> + Send>> + Clone {
    move |request: Request, next: Next| {
        Box::pin(async move {
            // Extrair claims
            let claims = request
                .extensions()
                .get::<Claims>()
                .ok_or_else(|| AppError::Unauthorized("Authentication required".to_string()))?
                .clone();

            // Rate limit mais restritivo: 10 req/min para endpoints sensíveis
            let key = format!("ratelimit:{}:strict", claims.tenant_id);

            // TODO: Usar cache real aqui
            // Por enquanto, apenas log
            tracing::debug!("Strict rate limit check for tenant: {}", claims.tenant_id);

            Ok(next.run(request).await)
        })
    }
}

/// Rate limiting por IP (antes da autenticação)
pub async fn ip_rate_limit_middleware(
    request: Request,
    next: Next,
) -> Result<Response, AppError> {
    // Extrair IP do request
    let ip = extract_client_ip(&request);

    // TODO: Implementar rate limit por IP
    // Útil para prevenir brute force em /login

    tracing::debug!("Request from IP: {}", ip);

    Ok(next.run(request).await)
}

/// Extrair IP do cliente
fn extract_client_ip(request: &Request) -> String {
    // Tentar headers de proxy
    if let Some(forwarded) = request.headers().get("X-Forwarded-For") {
        if let Ok(value) = forwarded.to_str() {
            if let Some(ip) = value.split(',').next() {
                return ip.trim().to_string();
            }
        }
    }

    if let Some(real_ip) = request.headers().get("X-Real-IP") {
        if let Ok(value) = real_ip.to_str() {
            return value.to_string();
        }
    }

    // Fallback para conexão direta
    "unknown".to_string()
}

/// Rate limit para autenticação (prevenir brute force)
pub struct AuthRateLimiter {
    cache: Arc<CacheManager>,
}

impl AuthRateLimiter {
    pub fn new(cache: Arc<CacheManager>) -> Self {
        Self { cache }
    }

    /// Verificar se IP pode tentar login
    pub async fn check_login_attempts(&self, ip: &str) -> Result<bool, AppError> {
        let key = format!("auth:attempts:{}", ip);
        let count = self.cache.incr_with_ttl(&key, Duration::from_secs(300)).await?; // 5 minutos

        // Máximo 10 tentativas em 5 minutos
        Ok(count <= 10)
    }

    /// Registrar tentativa de login falhada
    pub async fn record_failed_login(&self, ip: &str, email: &str) -> Result<(), AppError> {
        let key_ip = format!("auth:attempts:{}", ip);
        let key_email = format!("auth:attempts:email:{}", email);

        self.cache.incr_with_ttl(&key_ip, Duration::from_secs(300)).await?;
        self.cache.incr_with_ttl(&key_email, Duration::from_secs(900)).await?; // 15 min

        // Log para segurança
        tracing::warn!("Failed login attempt: ip={}, email={}", ip, email);

        Ok(())
    }

    /// Verificar se email está bloqueado por muitas tentativas
    pub async fn is_email_blocked(&self, email: &str) -> Result<bool, AppError> {
        let key = format!("auth:attempts:email:{}", email);

        if let Some(count) = self.cache.get::<i64>(&key).await? {
            return Ok(count > 20); // Bloquear após 20 tentativas
        }

        Ok(false)
    }

    /// Reset tentativas após login bem-sucedido
    pub async fn reset_attempts(&self, ip: &str, email: &str) -> Result<(), AppError> {
        let key_ip = format!("auth:attempts:{}", ip);
        let key_email = format!("auth:attempts:email:{}", email);

        self.cache.delete(&key_ip).await?;
        self.cache.delete(&key_email).await?;

        Ok(())
    }
}

/// Estatísticas de rate limiting
pub struct RateLimitStats {
    pub total_requests: u64,
    pub rejected_requests: u64,
    pub rejection_rate: f64,
}

impl RateLimitStats {
    /// Buscar estatísticas de rate limit de um tenant
    pub async fn for_tenant(
        cache: &CacheManager,
        tenant_id: &str,
    ) -> Result<Self, AppError> {
        // TODO: Implementar métricas reais
        Ok(Self {
            total_requests: 0,
            rejected_requests: 0,
            rejection_rate: 0.0,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_rate_limit_config() {
        let startup = RateLimitConfig::startup();
        assert_eq!(startup.requests_per_minute, 100);

        let business = RateLimitConfig::business();
        assert_eq!(business.requests_per_minute, 500);

        let enterprise = RateLimitConfig::enterprise();
        assert_eq!(enterprise.requests_per_minute, 2000);
    }

    #[test]
    fn test_plan_config() {
        let config = RateLimitConfig::from_plan("startup");
        assert_eq!(config.requests_per_minute, 100);

        let config = RateLimitConfig::from_plan("enterprise");
        assert_eq!(config.requests_per_minute, 2000);

        // Plano desconhecido usa startup
        let config = RateLimitConfig::from_plan("unknown");
        assert_eq!(config.requests_per_minute, 100);
    }
}
