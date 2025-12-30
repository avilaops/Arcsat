use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    pub server: ServerConfig,
    pub database: DatabaseConfig,
    pub redis: RedisConfig,
    pub market_intelligence: MarketIntelligenceConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServerConfig {
    pub host: String,
    pub port: u16,
    pub workers: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DatabaseConfig {
    pub url: String,
    pub max_connections: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RedisConfig {
    pub url: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MarketIntelligenceConfig {
    pub enabled: bool,
    pub proxy_enabled: bool,
    pub proxy_url: Option<String>,
    pub max_concurrent_jobs: usize,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            server: ServerConfig {
                host: "0.0.0.0".to_string(),
                port: 3000,
                workers: 4,
            },
            database: DatabaseConfig {
                url: "postgresql://arcsat:arcsat@localhost/arcsat".to_string(),
                max_connections: 10,
            },
            redis: RedisConfig {
                url: "redis://localhost:6379".to_string(),
            },
            market_intelligence: MarketIntelligenceConfig {
                enabled: true,
                proxy_enabled: false,
                proxy_url: None,
                max_concurrent_jobs: 5,
            },
        }
    }
}
