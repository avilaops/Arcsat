-- Market Intelligence Schema
-- Database para Arcsat Market Intelligence

-- Tabela de jobs de scraping
CREATE TABLE IF NOT EXISTS scraping_jobs (
    id UUID PRIMARY KEY,
    tenant_id UUID NOT NULL,
    marketplace VARCHAR(50) NOT NULL,
    search_query TEXT NOT NULL,
    category VARCHAR(255),
    max_pages INTEGER NOT NULL DEFAULT 5,
    priority SMALLINT NOT NULL DEFAULT 5,
    status VARCHAR(20) NOT NULL DEFAULT 'pending',
    created_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW(),
    started_at TIMESTAMP WITH TIME ZONE,
    completed_at TIMESTAMP WITH TIME ZONE,
    error TEXT,

    INDEX idx_tenant_id (tenant_id),
    INDEX idx_status (status),
    INDEX idx_created_at (created_at)
);

-- Tabela de produtos coletados
CREATE TABLE IF NOT EXISTS scraped_products (
    id UUID PRIMARY KEY,
    job_id UUID NOT NULL REFERENCES scraping_jobs(id) ON DELETE CASCADE,
    marketplace VARCHAR(50) NOT NULL,
    external_id VARCHAR(255) NOT NULL,
    title TEXT NOT NULL,
    price DECIMAL(10, 2) NOT NULL,
    currency VARCHAR(3) NOT NULL DEFAULT 'BRL',
    url TEXT NOT NULL,
    image_url TEXT,
    seller_name VARCHAR(255) NOT NULL,
    seller_id VARCHAR(255),
    seller_rating DECIMAL(3, 2),
    sales_rank INTEGER,
    rating DECIMAL(3, 2),
    num_reviews INTEGER NOT NULL DEFAULT 0,
    availability BOOLEAN NOT NULL DEFAULT true,
    category VARCHAR(255),
    brand VARCHAR(255),
    scraped_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW(),
    extra JSONB,

    INDEX idx_job_id (job_id),
    INDEX idx_marketplace (marketplace),
    INDEX idx_external_id (external_id),
    INDEX idx_price (price),
    INDEX idx_scraped_at (scraped_at),
    UNIQUE INDEX idx_job_external (job_id, external_id)
);

-- Tabela de análises de tendência
CREATE TABLE IF NOT EXISTS trend_analyses (
    id UUID PRIMARY KEY,
    tenant_id UUID NOT NULL,
    marketplace VARCHAR(50) NOT NULL,
    category VARCHAR(255) NOT NULL,
    period_start TIMESTAMP WITH TIME ZONE NOT NULL,
    period_end TIMESTAMP WITH TIME ZONE NOT NULL,
    total_products BIGINT NOT NULL,
    avg_price DECIMAL(10, 2) NOT NULL,
    median_price DECIMAL(10, 2) NOT NULL,
    min_price DECIMAL(10, 2) NOT NULL,
    max_price DECIMAL(10, 2) NOT NULL,
    top_sellers JSONB NOT NULL,
    trending_keywords JSONB NOT NULL,
    growth_rate DECIMAL(5, 2) NOT NULL DEFAULT 0.0,
    competition_level VARCHAR(20) NOT NULL,
    analyzed_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW(),

    INDEX idx_tenant_id (tenant_id),
    INDEX idx_marketplace (marketplace),
    INDEX idx_category (category),
    INDEX idx_analyzed_at (analyzed_at)
);

-- Tabela de configurações de proxy
CREATE TABLE IF NOT EXISTS proxy_configs (
    id SERIAL PRIMARY KEY,
    tenant_id UUID NOT NULL,
    name VARCHAR(255) NOT NULL,
    url TEXT NOT NULL,
    username VARCHAR(255),
    password VARCHAR(255),
    proxy_type VARCHAR(20) NOT NULL DEFAULT 'http',
    enabled BOOLEAN NOT NULL DEFAULT true,
    last_used TIMESTAMP WITH TIME ZONE,
    success_rate DECIMAL(5, 2) DEFAULT 100.0,
    created_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW(),

    INDEX idx_tenant_id (tenant_id),
    INDEX idx_enabled (enabled)
);

-- Tabela de alertas de oportunidades
CREATE TABLE IF NOT EXISTS market_alerts (
    id UUID PRIMARY KEY,
    tenant_id UUID NOT NULL,
    marketplace VARCHAR(50) NOT NULL,
    alert_type VARCHAR(50) NOT NULL, -- 'price_drop', 'high_demand', 'low_competition'
    product_id UUID REFERENCES scraped_products(id),
    title TEXT NOT NULL,
    message TEXT NOT NULL,
    severity VARCHAR(20) NOT NULL DEFAULT 'info', -- 'info', 'warning', 'critical'
    read_at TIMESTAMP WITH TIME ZONE,
    created_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW(),

    INDEX idx_tenant_id (tenant_id),
    INDEX idx_created_at (created_at),
    INDEX idx_read_at (read_at)
);

-- Views úteis

-- View: Jobs recentes por tenant
CREATE OR REPLACE VIEW recent_jobs AS
SELECT
    j.id,
    j.tenant_id,
    j.marketplace,
    j.search_query,
    j.status,
    j.created_at,
    COUNT(p.id) as products_count
FROM scraping_jobs j
LEFT JOIN scraped_products p ON j.id = p.job_id
WHERE j.created_at > NOW() - INTERVAL '30 days'
GROUP BY j.id
ORDER BY j.created_at DESC;

-- View: Preços médios por categoria
CREATE OR REPLACE VIEW category_pricing AS
SELECT
    marketplace,
    category,
    AVG(price) as avg_price,
    MIN(price) as min_price,
    MAX(price) as max_price,
    COUNT(*) as product_count,
    MAX(scraped_at) as last_updated
FROM scraped_products
WHERE scraped_at > NOW() - INTERVAL '7 days'
GROUP BY marketplace, category;
