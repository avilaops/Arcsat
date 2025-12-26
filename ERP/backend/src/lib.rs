// lib.rs - Biblioteca principal do ERP/CRM

pub mod auth;
pub mod cache;
pub mod db;
pub mod error;
// Módulos temporariamente desabilitados para compilação
// pub mod middleware;
pub mod models;
pub mod mongodb;
// pub mod routes;
// pub mod webhooks;

// Re-exports para facilitar uso
pub use auth::{AuthConfig, Claims};
pub use db::DbPool;
pub use error::{AppError, Result};
