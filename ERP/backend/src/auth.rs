// auth.rs - Módulo de Autenticação Enterprise

use axum::{
    async_trait,
    extract::{FromRequestParts, Query, State},
    http::{header, request::Parts, StatusCode},
    response::{IntoResponse, Redirect, Response},
    Json,
};
use chrono::{Duration, Utc};
use jsonwebtoken::{decode, encode, DecodingKey, EncodingKey, Header, Validation};
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use std::sync::Arc;

use crate::error::AppError;

// ============================================================================
// ESTRUTURAS DE DADOS
// ============================================================================

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Claims {
    pub sub: String,           // User ID
    pub tenant_id: String,     // Tenant ID
    pub email: String,
    pub roles: Vec<String>,
    pub permissions: Vec<String>,
    pub exp: i64,              // Expiration timestamp
    pub iat: i64,              // Issued at timestamp
    pub jti: String,           // JWT ID (para revogação)
}

#[derive(Debug, Serialize, Deserialize)]
pub struct LoginRequest {
    pub email: String,
    pub password: String,
    pub tenant_domain: String,
    #[serde(default)]
    pub mfa_token: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct LoginResponse {
    pub access_token: String,
    pub refresh_token: String,
    pub expires_in: i64,
    pub user: UserInfo,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub requires_mfa: Option<bool>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct UserInfo {
    pub id: String,
    pub name: String,
    pub email: String,
    pub roles: Vec<String>,
    pub tenant_id: String,
    pub avatar_url: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct RefreshRequest {
    pub refresh_token: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct MFASetupRequest {
    pub method: MFAMethod,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum MFAMethod {
    TOTP,    // Google Authenticator, etc
    SMS,
    Email,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct MFASetupResponse {
    pub secret: String,
    pub qr_code: String,
    pub backup_codes: Vec<String>,
}

// Estado de autenticação
#[derive(Clone)]
pub struct AuthConfig {
    pub jwt_secret: String,
    pub jwt_expiration_hours: i64,
    pub refresh_token_days: i64,
}

// ============================================================================
// JWT HANDLER
// ============================================================================

impl AuthConfig {
    pub fn new() -> Self {
        Self {
            jwt_secret: std::env::var("JWT_SECRET")
                .expect("JWT_SECRET must be set"),
            jwt_expiration_hours: 8,
            refresh_token_days: 30,
        }
    }

    /// Gerar access token JWT
    pub fn generate_access_token(&self, user_id: &str, tenant_id: &str, email: &str, roles: Vec<String>, permissions: Vec<String>) -> Result<String, AppError> {
        let expiration = Utc::now()
            .checked_add_signed(Duration::hours(self.jwt_expiration_hours))
            .expect("valid timestamp")
            .timestamp();

        let claims = Claims {
            sub: user_id.to_string(),
            tenant_id: tenant_id.to_string(),
            email: email.to_string(),
            roles,
            permissions,
            exp: expiration,
            iat: Utc::now().timestamp(),
            jti: Uuid::new_v4().to_string(),
        };

        encode(
            &Header::default(),
            &claims,
            &EncodingKey::from_secret(self.jwt_secret.as_ref()),
        )
        .map_err(|e| AppError::InternalError(format!("Failed to generate token: {}", e)))
    }

    /// Gerar refresh token
    pub fn generate_refresh_token(&self, user_id: &str) -> Result<String, AppError> {
        let expiration = Utc::now()
            .checked_add_signed(Duration::days(self.refresh_token_days))
            .expect("valid timestamp")
            .timestamp();

        let claims = Claims {
            sub: user_id.to_string(),
            tenant_id: "".to_string(), // Refresh token não tem tenant
            email: "".to_string(),
            roles: vec![],
            permissions: vec![],
            exp: expiration,
            iat: Utc::now().timestamp(),
            jti: Uuid::new_v4().to_string(),
        };

        encode(
            &Header::default(),
            &claims,
            &EncodingKey::from_secret(self.jwt_secret.as_ref()),
        )
        .map_err(|e| AppError::InternalError(format!("Failed to generate refresh token: {}", e)))
    }

    /// Validar e decodificar token
    pub fn validate_token(&self, token: &str) -> Result<Claims, AppError> {
        decode::<Claims>(
            token,
            &DecodingKey::from_secret(self.jwt_secret.as_ref()),
            &Validation::default(),
        )
        .map(|data| data.claims)
        .map_err(|e| AppError::Unauthorized(format!("Invalid token: {}", e)))
    }
}

// ============================================================================
// EXTRACTOR PARA AUTENTICAÇÃO
// ============================================================================

/// Extrator que valida JWT automaticamente
#[async_trait]
impl<S> FromRequestParts<S> for Claims
where
    S: Send + Sync,
{
    type Rejection = AppError;

    async fn from_request_parts(parts: &mut Parts, _state: &S) -> Result<Self, Self::Rejection> {
        // Extrair token do header Authorization
        let auth_header = parts
            .headers
            .get(header::AUTHORIZATION)
            .and_then(|value| value.to_str().ok())
            .ok_or_else(|| AppError::Unauthorized("Missing authorization header".to_string()))?;

        // Verificar formato "Bearer {token}"
        let token = auth_header
            .strip_prefix("Bearer ")
            .ok_or_else(|| AppError::Unauthorized("Invalid authorization header format".to_string()))?;

        // Validar token
        let config = AuthConfig::new();
        config.validate_token(token)
    }
}

// ============================================================================
// HANDLERS DE AUTENTICAÇÃO
// ============================================================================

/// Login com email/senha
pub async fn login(
    State(config): State<Arc<AuthConfig>>,
    Json(payload): Json<LoginRequest>,
) -> Result<Json<LoginResponse>, AppError> {
    // TODO: Buscar usuário no banco de dados
    // TODO: Validar senha com bcrypt/argon2
    // TODO: Verificar se MFA está habilitado
    // TODO: Validar tenant_domain
    
    // Simulação (remover em produção)
    let user_id = Uuid::new_v4().to_string();
    let tenant_id = Uuid::new_v4().to_string();
    let roles = vec!["admin".to_string(), "sales_manager".to_string()];
    let permissions = vec!["crm.*".to_string(), "finance.read".to_string()];

    // Verificar senha
    let password_valid = verify_password(&payload.password, "$2b$12$LQv3c1yqBWVHxkd0LHAkCOYz6TtxMQJqhN8/LewY5GyDK0Y8qKQBu")?; // hash de exemplo
    
    if !password_valid {
        return Err(AppError::Unauthorized("Invalid credentials".to_string()));
    }

    // Gerar tokens
    let access_token = config.generate_access_token(
        &user_id,
        &tenant_id,
        &payload.email,
        roles.clone(),
        permissions,
    )?;

    let refresh_token = config.generate_refresh_token(&user_id)?;

    Ok(Json(LoginResponse {
        access_token,
        refresh_token,
        expires_in: config.jwt_expiration_hours * 3600,
        user: UserInfo {
            id: user_id,
            name: "João Silva".to_string(),
            email: payload.email,
            roles,
            tenant_id,
            avatar_url: None,
        },
        requires_mfa: None,
    }))
}

/// Refresh token
pub async fn refresh(
    State(config): State<Arc<AuthConfig>>,
    Json(payload): Json<RefreshRequest>,
) -> Result<Json<LoginResponse>, AppError> {
    // Validar refresh token
    let claims = config.validate_token(&payload.refresh_token)?;

    // TODO: Verificar se refresh token não foi revogado (Redis)
    // TODO: Buscar dados atualizados do usuário

    // Gerar novo access token
    let access_token = config.generate_access_token(
        &claims.sub,
        &claims.tenant_id,
        &claims.email,
        claims.roles.clone(),
        claims.permissions.clone(),
    )?;

    Ok(Json(LoginResponse {
        access_token,
        refresh_token: payload.refresh_token, // Manter o mesmo refresh token
        expires_in: config.jwt_expiration_hours * 3600,
        user: UserInfo {
            id: claims.sub,
            name: "João Silva".to_string(),
            email: claims.email,
            roles: claims.roles,
            tenant_id: claims.tenant_id,
            avatar_url: None,
        },
        requires_mfa: None,
    }))
}

/// Logout (revogar token)
pub async fn logout(claims: Claims) -> Result<StatusCode, AppError> {
    // TODO: Adicionar JWT ID na blacklist do Redis com TTL = tempo restante do token
    tracing::info!("User {} logged out", claims.sub);
    Ok(StatusCode::NO_CONTENT)
}

// ============================================================================
// SSO - GOOGLE
// ============================================================================

#[derive(Debug, Deserialize)]
pub struct OAuthCallbackQuery {
    pub code: String,
    pub state: Option<String>,
}

/// Iniciar fluxo SSO Google
pub async fn google_sso_start() -> Result<Redirect, AppError> {
    // TODO: Implementar OAuth2 com google-oauth crate
    let google_auth_url = "https://accounts.google.com/o/oauth2/v2/auth";
    let client_id = std::env::var("GOOGLE_CLIENT_ID")
        .map_err(|_| AppError::InternalError("GOOGLE_CLIENT_ID not configured".to_string()))?;
    
    let redirect_uri = "http://localhost:3000/api/v1/auth/sso/google/callback";
    let state = Uuid::new_v4().to_string(); // Guardar no Redis para validação

    let auth_url = format!(
        "{}?client_id={}&redirect_uri={}&response_type=code&scope=email profile&state={}",
        google_auth_url, client_id, redirect_uri, state
    );

    Ok(Redirect::to(&auth_url))
}

/// Callback SSO Google
pub async fn google_sso_callback(
    Query(query): Query<OAuthCallbackQuery>,
    State(config): State<Arc<AuthConfig>>,
) -> Result<Json<LoginResponse>, AppError> {
    // TODO: Validar state (CSRF protection)
    // TODO: Trocar code por access_token
    // TODO: Buscar perfil do usuário
    // TODO: Criar ou atualizar usuário no banco
    
    // Simulação
    let user_id = Uuid::new_v4().to_string();
    let tenant_id = Uuid::new_v4().to_string();
    
    let access_token = config.generate_access_token(
        &user_id,
        &tenant_id,
        "user@example.com",
        vec!["user".to_string()],
        vec!["crm.read".to_string()],
    )?;

    let refresh_token = config.generate_refresh_token(&user_id)?;

    Ok(Json(LoginResponse {
        access_token,
        refresh_token,
        expires_in: config.jwt_expiration_hours * 3600,
        user: UserInfo {
            id: user_id,
            name: "Google User".to_string(),
            email: "user@example.com".to_string(),
            roles: vec!["user".to_string()],
            tenant_id,
            avatar_url: None,
        },
        requires_mfa: None,
    }))
}

// ============================================================================
// MFA - MULTI-FACTOR AUTHENTICATION
// ============================================================================

/// Configurar MFA para usuário
pub async fn setup_mfa(
    claims: Claims,
    Json(payload): Json<MFASetupRequest>,
) -> Result<Json<MFASetupResponse>, AppError> {
    match payload.method {
        MFAMethod::TOTP => {
            // TODO: Gerar secret TOTP
            // TODO: Gerar QR code
            // TODO: Gerar backup codes
            // TODO: Salvar no banco
            
            Ok(Json(MFASetupResponse {
                secret: "BASE32ENCODEDSECRET".to_string(),
                qr_code: "data:image/png;base64,...".to_string(),
                backup_codes: vec![
                    "ABCD-1234".to_string(),
                    "EFGH-5678".to_string(),
                ],
            }))
        }
        MFAMethod::SMS | MFAMethod::Email => {
            // TODO: Implementar envio de código via SMS/Email
            Err(AppError::BadRequest("Method not implemented yet".to_string()))
        }
    }
}

/// Verificar código MFA
pub async fn verify_mfa(
    claims: Claims,
    Json(code): Json<String>,
) -> Result<StatusCode, AppError> {
    // TODO: Validar código TOTP ou backup code
    Ok(StatusCode::OK)
}

// ============================================================================
// HELPERS
// ============================================================================

/// Verificar senha com bcrypt
fn verify_password(password: &str, hash: &str) -> Result<bool, AppError> {
    bcrypt::verify(password, hash)
        .map_err(|e| AppError::InternalError(format!("Password verification failed: {}", e)))
}

/// Hash de senha com bcrypt
pub fn hash_password(password: &str) -> Result<String, AppError> {
    bcrypt::hash(password, bcrypt::DEFAULT_COST)
        .map_err(|e| AppError::InternalError(format!("Password hashing failed: {}", e)))
}

// ============================================================================
// ROTAS
// ============================================================================

pub fn auth_routes() -> axum::Router<Arc<AuthConfig>> {
    use axum::routing::{get, post};

    axum::Router::new()
        .route("/login", post(login))
        .route("/refresh", post(refresh))
        .route("/logout", post(logout))
        .route("/sso/google", get(google_sso_start))
        .route("/sso/google/callback", get(google_sso_callback))
        .route("/mfa/setup", post(setup_mfa))
        .route("/mfa/verify", post(verify_mfa))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_generate_and_validate_token() {
        let config = AuthConfig {
            jwt_secret: "test_secret_key_min_32_chars_long".to_string(),
            jwt_expiration_hours: 8,
            refresh_token_days: 30,
        };

        let token = config
            .generate_access_token(
                "user-123",
                "tenant-456",
                "test@example.com",
                vec!["admin".to_string()],
                vec!["*".to_string()],
            )
            .unwrap();

        let claims = config.validate_token(&token).unwrap();
        assert_eq!(claims.sub, "user-123");
        assert_eq!(claims.tenant_id, "tenant-456");
        assert_eq!(claims.email, "test@example.com");
    }

    #[test]
    fn test_password_hashing() {
        let password = "my_secure_password";
        let hash = hash_password(password).unwrap();
        assert!(verify_password(password, &hash).unwrap());
        assert!(!verify_password("wrong_password", &hash).unwrap());
    }
}
