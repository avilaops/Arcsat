// middleware/rbac.rs - Role-Based Access Control

use axum::{
    extract::Request,
    http::StatusCode,
    middleware::Next,
    response::Response,
};
use std::collections::HashSet;

use crate::{
    auth::Claims,
    error::AppError,
};

/// Estrutura de permissão
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Permission {
    pub resource: String,  // "crm", "finance", "hr"
    pub action: Action,    // Read, Write, Delete, etc
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum Action {
    Read,
    Write,
    Delete,
    Approve,
    Admin,
}

impl Action {
    pub fn from_str(s: &str) -> Option<Self> {
        match s {
            "read" => Some(Action::Read),
            "write" => Some(Action::Write),
            "delete" => Some(Action::Delete),
            "approve" => Some(Action::Approve),
            "admin" => Some(Action::Admin),
            _ => None,
        }
    }
}

/// Middleware para verificar permissões
/// 
/// Uso:
/// ```
/// .route("/leads", post(create_lead))
///     .layer(middleware::from_fn(require_permission("crm", Action::Write)))
/// ```
pub fn require_permission(resource: &'static str, action: Action) -> impl Fn(Request, Next) -> std::pin::Pin<Box<dyn std::future::Future<Output = Result<Response, AppError>> + Send>> + Clone {
    move |request: Request, next: Next| {
        let resource = resource.to_string();
        let action = action.clone();
        
        Box::pin(async move {
            // Extrair claims do request (já validado pelo auth middleware)
            let claims = request
                .extensions()
                .get::<Claims>()
                .ok_or_else(|| AppError::Unauthorized("Authentication required".to_string()))?
                .clone();

            // Verificar se usuário tem a permissão
            if !has_permission(&claims, &resource, &action) {
                tracing::warn!(
                    "Permission denied: user={}, resource={}, action={:?}",
                    claims.sub,
                    resource,
                    action
                );
                return Err(AppError::Forbidden(
                    format!("You don't have permission to {} {}", format!("{:?}", action).to_lowercase(), resource)
                ));
            }

            Ok(next.run(request).await)
        })
    }
}

/// Verificar se usuário tem permissão
fn has_permission(claims: &Claims, resource: &str, action: &Action) -> bool {
    // Admin tem acesso total
    if claims.roles.contains(&"admin".to_string()) {
        return true;
    }

    // Verificar permissões explícitas
    for permission in &claims.permissions {
        if permission_matches(permission, resource, action) {
            return true;
        }
    }

    false
}

/// Verificar se string de permissão match com resource e action
/// 
/// Exemplos:
/// - "*" -> tudo
/// - "crm.*" -> tudo em CRM
/// - "crm.read" -> apenas leitura em CRM
/// - "finance.approve" -> aprovar em financeiro
fn permission_matches(permission_str: &str, resource: &str, action: &Action) -> bool {
    // Wildcard total
    if permission_str == "*" {
        return true;
    }

    let parts: Vec<&str> = permission_str.split('.').collect();
    
    if parts.len() != 2 {
        return false;
    }

    let perm_resource = parts[0];
    let perm_action = parts[1];

    // Verificar resource
    if perm_resource != "*" && perm_resource != resource {
        return false;
    }

    // Verificar action
    if perm_action == "*" {
        return true;
    }

    if let Some(required_action) = Action::from_str(perm_action) {
        return &required_action == action;
    }

    false
}

/// Middleware para verificar roles
pub fn require_role(required_role: &'static str) -> impl Fn(Request, Next) -> std::pin::Pin<Box<dyn std::future::Future<Output = Result<Response, AppError>> + Send>> + Clone {
    move |request: Request, next: Next| {
        let required_role = required_role.to_string();
        
        Box::pin(async move {
            let claims = request
                .extensions()
                .get::<Claims>()
                .ok_or_else(|| AppError::Unauthorized("Authentication required".to_string()))?
                .clone();

            if !claims.roles.contains(&required_role) {
                tracing::warn!(
                    "Role required: user={}, required_role={}",
                    claims.sub,
                    required_role
                );
                return Err(AppError::Forbidden(
                    format!("This action requires '{}' role", required_role)
                ));
            }

            Ok(next.run(request).await)
        })
    }
}

/// Verificar se usuário pode acessar recurso de outro usuário
/// (útil para /leads/:id onde owner_id != current_user)
pub fn can_access_resource(claims: &Claims, owner_id: &str, resource_tenant_id: &str) -> bool {
    // Admin pode acessar tudo do tenant
    if claims.roles.contains(&"admin".to_string()) {
        return true;
    }

    // Tenant deve ser o mesmo
    if claims.tenant_id != resource_tenant_id {
        return false;
    }

    // Recurso pertence ao usuário
    if claims.sub == owner_id {
        return true;
    }

    // Manager pode ver recursos do time
    if claims.roles.contains(&"manager".to_string()) {
        // TODO: Verificar hierarquia no banco
        return true;
    }

    false
}

/// Definição de roles padrão e suas permissões
pub struct RoleDefinitions;

impl RoleDefinitions {
    /// Admin - acesso total
    pub fn admin_permissions() -> Vec<String> {
        vec!["*".to_string()]
    }

    /// Sales Manager - gerenciar CRM e ver analytics
    pub fn sales_manager_permissions() -> Vec<String> {
        vec![
            "crm.*".to_string(),
            "analytics.read".to_string(),
            "reports.read".to_string(),
        ]
    }

    /// Sales Rep - CRM básico
    pub fn sales_rep_permissions() -> Vec<String> {
        vec![
            "crm.read".to_string(),
            "crm.write".to_string(),
            "leads.read".to_string(),
            "leads.write".to_string(),
            "activities.read".to_string(),
            "activities.write".to_string(),
        ]
    }

    /// Finance Manager - gestão financeira completa
    pub fn finance_manager_permissions() -> Vec<String> {
        vec![
            "finance.*".to_string(),
            "reports.read".to_string(),
        ]
    }

    /// Finance Analyst - leitura + contas a pagar
    pub fn finance_analyst_permissions() -> Vec<String> {
        vec![
            "finance.read".to_string(),
            "accounts_payable.write".to_string(),
            "accounts_receivable.read".to_string(),
        ]
    }

    /// HR Manager - gestão completa de RH
    pub fn hr_manager_permissions() -> Vec<String> {
        vec![
            "hr.*".to_string(),
            "employees.*".to_string(),
            "payroll.*".to_string(),
        ]
    }

    /// HR Analyst - visualização e básico
    pub fn hr_analyst_permissions() -> Vec<String> {
        vec![
            "hr.read".to_string(),
            "employees.read".to_string(),
            "attendance.write".to_string(),
        ]
    }

    /// User - acesso básico (ver próprios dados)
    pub fn user_permissions() -> Vec<String> {
        vec![
            "profile.read".to_string(),
            "profile.write".to_string(),
        ]
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_permission_matching() {
        // Wildcard total
        assert!(permission_matches("*", "crm", &Action::Read));
        assert!(permission_matches("*", "finance", &Action::Write));

        // Wildcard por recurso
        assert!(permission_matches("crm.*", "crm", &Action::Read));
        assert!(permission_matches("crm.*", "crm", &Action::Write));
        assert!(!permission_matches("crm.*", "finance", &Action::Read));

        // Permissão específica
        assert!(permission_matches("crm.read", "crm", &Action::Read));
        assert!(!permission_matches("crm.read", "crm", &Action::Write));
        assert!(!permission_matches("crm.read", "finance", &Action::Read));
    }

    #[test]
    fn test_has_permission() {
        let claims = Claims {
            sub: "user-123".to_string(),
            tenant_id: "tenant-456".to_string(),
            email: "user@example.com".to_string(),
            roles: vec!["sales_rep".to_string()],
            permissions: vec!["crm.read".to_string(), "crm.write".to_string()],
            exp: 9999999999,
            iat: 1234567890,
            jti: "jwt-id".to_string(),
        };

        assert!(has_permission(&claims, "crm", &Action::Read));
        assert!(has_permission(&claims, "crm", &Action::Write));
        assert!(!has_permission(&claims, "crm", &Action::Delete));
        assert!(!has_permission(&claims, "finance", &Action::Read));
    }

    #[test]
    fn test_admin_has_all_permissions() {
        let claims = Claims {
            sub: "admin-123".to_string(),
            tenant_id: "tenant-456".to_string(),
            email: "admin@example.com".to_string(),
            roles: vec!["admin".to_string()],
            permissions: vec![],
            exp: 9999999999,
            iat: 1234567890,
            jti: "jwt-id".to_string(),
        };

        assert!(has_permission(&claims, "crm", &Action::Read));
        assert!(has_permission(&claims, "finance", &Action::Delete));
        assert!(has_permission(&claims, "hr", &Action::Admin));
    }
}
