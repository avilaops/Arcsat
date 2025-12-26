// middleware/tenant.rs - Multi-tenancy Middleware

use axum::{
    extract::{Request, State},
    http::{header, StatusCode},
    middleware::Next,
    response::{IntoResponse, Response},
};
use std::sync::Arc;
use uuid::Uuid;

use crate::{
    db::DbPool,
    error::AppError,
};

/// Extrator de Tenant ID do request
#[derive(Debug, Clone)]
pub struct TenantContext {
    pub tenant_id: Uuid,
    pub tenant_domain: String,
}

/// Middleware para extrair e validar tenant
/// 
/// Pode extrair tenant de:
/// 1. Subdomain (tenant.erp.com)
/// 2. Header X-Tenant-ID
/// 3. JWT Claims (já validado)
pub async fn tenant_middleware(
    State(pool): State<Arc<DbPool>>,
    mut request: Request,
    next: Next,
) -> Result<Response, AppError> {
    // Tentar extrair tenant_id do domínio
    let tenant_domain = extract_tenant_from_host(&request)?;
    
    // Buscar tenant no banco
    let tenant = sqlx::query!(
        r#"
        SELECT id, name, domain, status, plan
        FROM tenants
        WHERE domain = $1 AND status = 'active'
        "#,
        tenant_domain
    )
    .fetch_optional(pool.as_ref())
    .await
    .map_err(|e| AppError::InternalError(format!("Database error: {}", e)))?
    .ok_or_else(|| AppError::not_found("Tenant"))?;

    // Verificar se tenant está ativo
    if tenant.status != "active" {
        return Err(AppError::Forbidden(
            "Tenant account is suspended or inactive".to_string()
        ));
    }

    // Adicionar tenant context nas extensions do request
    request.extensions_mut().insert(TenantContext {
        tenant_id: tenant.id,
        tenant_domain: tenant.domain,
    });

    tracing::debug!("Request from tenant: {} ({})", tenant.name, tenant.id);

    Ok(next.run(request).await)
}

/// Extrair tenant do Host header
fn extract_tenant_from_host(request: &Request) -> Result<String, AppError> {
    let host = request
        .headers()
        .get(header::HOST)
        .and_then(|h| h.to_str().ok())
        .ok_or_else(|| AppError::BadRequest("Missing Host header".to_string()))?;

    // Exemplo: empresa.erp.com -> empresa
    // Ou: localhost:3000 -> default (dev)
    
    if host.starts_with("localhost") || host.starts_with("127.0.0.1") {
        // Ambiente de desenvolvimento
        return Ok("default".to_string());
    }

    // Extrair subdomain
    let parts: Vec<&str> = host.split('.').collect();
    if parts.len() >= 2 {
        Ok(parts[0].to_string())
    } else {
        Err(AppError::BadRequest("Invalid host format".to_string()))
    }
}

/// Middleware alternativo: extrair tenant de header customizado
pub async fn tenant_from_header_middleware(
    State(pool): State<Arc<DbPool>>,
    mut request: Request,
    next: Next,
) -> Result<Response, AppError> {
    let tenant_id_str = request
        .headers()
        .get("X-Tenant-ID")
        .and_then(|h| h.to_str().ok())
        .ok_or_else(|| AppError::BadRequest("Missing X-Tenant-ID header".to_string()))?;

    let tenant_id = Uuid::parse_str(tenant_id_str)
        .map_err(|_| AppError::BadRequest("Invalid tenant ID format".to_string()))?;

    // Buscar tenant
    let tenant = sqlx::query!(
        r#"
        SELECT id, name, domain, status
        FROM tenants
        WHERE id = $1 AND status = 'active'
        "#,
        tenant_id
    )
    .fetch_optional(pool.as_ref())
    .await
    .map_err(|e| AppError::InternalError(format!("Database error: {}", e)))?
    .ok_or_else(|| AppError::not_found("Tenant"))?;

    request.extensions_mut().insert(TenantContext {
        tenant_id: tenant.id,
        tenant_domain: tenant.domain,
    });

    Ok(next.run(request).await)
}

/// Middleware para validar limites do tenant (rate limiting, storage, users)
pub async fn tenant_limits_middleware(
    State(pool): State<Arc<DbPool>>,
    request: Request,
    next: Next,
) -> Result<Response, AppError> {
    let tenant_ctx = request
        .extensions()
        .get::<TenantContext>()
        .ok_or_else(|| AppError::InternalError("Tenant context not found".to_string()))?;

    // Buscar configurações do tenant
    let tenant = sqlx::query!(
        r#"
        SELECT max_users, storage_limit_gb, plan
        FROM tenants
        WHERE id = $1
        "#,
        tenant_ctx.tenant_id
    )
    .fetch_one(pool.as_ref())
    .await
    .map_err(|e| AppError::InternalError(format!("Database error: {}", e)))?;

    // TODO: Validar limites
    // - Número de usuários ativos
    // - Uso de storage
    // - Rate limits por plano

    Ok(next.run(request).await)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extract_tenant_from_subdomain() {
        // Simular request com Host header
        // TODO: Implementar testes
    }
}
