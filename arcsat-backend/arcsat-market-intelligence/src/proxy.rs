use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProxyConfig {
    pub url: String,
    pub username: Option<String>,
    pub password: Option<String>,
    pub proxy_type: ProxyType,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ProxyType {
    Http,
    Https,
    Socks5,
}

impl ProxyConfig {
    pub fn new(url: impl Into<String>) -> Self {
        Self {
            url: url.into(),
            username: None,
            password: None,
            proxy_type: ProxyType::Http,
        }
    }

    pub fn with_auth(mut self, username: impl Into<String>, password: impl Into<String>) -> Self {
        self.username = Some(username.into());
        self.password = Some(password.into());
        self
    }

    pub fn with_type(mut self, proxy_type: ProxyType) -> Self {
        self.proxy_type = proxy_type;
        self
    }
}

/// Pool de proxies rotativos
pub struct ProxyPool {
    proxies: Vec<ProxyConfig>,
    current: usize,
}

impl ProxyPool {
    pub fn new(proxies: Vec<ProxyConfig>) -> Self {
        Self {
            proxies,
            current: 0,
        }
    }

    pub fn next(&mut self) -> Option<&ProxyConfig> {
        if self.proxies.is_empty() {
            return None;
        }

        let proxy = &self.proxies[self.current];
        self.current = (self.current + 1) % self.proxies.len();
        Some(proxy)
    }

    pub fn count(&self) -> usize {
        self.proxies.len()
    }
}
