# 🏢 Arcsat Backend - ERP/CRM com Market Intelligence

## 🎯 Arquitetura

```
arcsat-backend/
├── arcsat-core/              # Tipos compartilhados, errors, config
├── arcsat-market-intelligence/  # Módulo de scraping e análise
├── arcsat-erp/               # Módulo ERP (TODO)
├── arcsat-crm/               # Módulo CRM (TODO)
└── arcsat-api/               # Servidor HTTP principal
```

## 🚀 Quick Start

### 1. Setup

```bash
cd arcsat-backend

# Instalar Chrome/Chromium para headless scraping
# Windows: https://www.google.com/chrome/
# Linux: sudo apt-get install chromium-browser

# Copiar .env
cp .env.example .env
# Editar conforme necessário
```

### 2. Rodar com Docker Compose (Recomendado)

```bash
# Na raiz do Arcsat
docker-compose up -d postgres redis

# Ou criar o docker-compose completo:
cd arcsat-backend
cargo build --release
./target/release/arcsat-server
```

### 3. Rodar local (desenvolvimento)

```bash
# Terminal 1: PostgreSQL
docker run -d -p 5432:5432 -e POSTGRES_PASSWORD=arcsat postgres:15-alpine

# Terminal 2: Redis
docker run -d -p 6379:6379 redis:7-alpine

# Terminal 3: Backend
cd arcsat-backend
cargo run --bin arcsat-server
```

## 📡 API Endpoints

### Health Check
```bash
curl http://localhost:3000/health
```

### Market Intelligence

#### Criar Job de Scraping
```bash
curl -X POST http://localhost:3000/api/v1/market-intelligence/jobs \
  -H "Content-Type: application/json" \
  -d '{
    "marketplace": "mercado_livre",
    "search_query": "notebook gamer",
    "max_pages": 3,
    "priority": 5
  }'

# Response:
# {
#   "success": true,
#   "data": {
#     "job_id": "uuid",
#     "status": "pending",
#     "message": "Job criado e enfileirado"
#   }
# }
```

#### Verificar Status do Job
```bash
curl http://localhost:3000/api/v1/market-intelligence/jobs/{job_id}/status
```

#### Obter Análise de Tendências
```bash
curl "http://localhost:3000/api/v1/market-intelligence/trends?marketplace=amazon&category=eletronicos"
```

## 🔧 Configuração de Proxies (Produção)

Para evitar bloqueios em produção, você precisa de proxies residenciais:

### Bright Data
```env
MI_PROXY_ENABLED=true
MI_PROXY_URL=http://brd-customer-xxxxx:password@brd.superproxy.io:22225
```

### Oxylabs
```env
MI_PROXY_ENABLED=true
MI_PROXY_URL=http://customer-username:password@pr.oxylabs.io:7777
```

### ScraperAPI (mais simples)
```env
MI_PROXY_ENABLED=true
MI_PROXY_URL=http://scraperapi:YOUR_API_KEY@proxy-server.scraperapi.com:8001
```

## 🏗️ Arquitetura do Market Intelligence

### Componentes

1. **Scrapers** (`scrapers.rs`)
   - Amazon BR, Mercado Livre (implementados)
   - B2W, Magalu, Shopee, AliExpress (stubs)
   - Headless Chrome com stealth
   - Rotação de User-Agents

2. **Queue** (`queue.rs`)
   - Redis para gerenciamento de jobs
   - Filas por prioridade (1-10)
   - Status tracking

3. **Analysis** (`analysis.rs`)
   - Extração de keywords
   - Análise de preços (média, mediana, min, max)
   - Top sellers
   - Nível de competição

4. **Proxy** (`proxy.rs`)
   - Suporte a proxies rotativos
   - Pool management

## 📊 Integração com o ERP

O módulo de Market Intelligence está integrado ao ERP através de:

1. **Multi-tenancy**: Cada scraping é associado a um `tenant_id`
2. **CRM**: Análises podem alimentar estratégias de vendas
3. **Pricing**: Insights de preços competitivos
4. **Estoque**: Identificar produtos em alta demanda

### Exemplo de Integração

```rust
// No módulo CRM
use arcsat_market_intelligence::MarketIntelligenceEngine;

// Criar análise de mercado para um produto do CRM
let job = ScrapingJob::new(
    tenant_id,
    Marketplace::Amazon,
    "produto similar ao nosso".to_string(),
    5
);

let job_id = mi_engine.submit_job(job).await?;

// Aguardar conclusão e obter insights
let products = mi_engine.get_results(&job_id).await?;
let analysis = mi_engine.analysis.analyze(tenant_id, marketplace, category, &products);

// Usar insights no CRM
crm.update_pricing_strategy(product_id, analysis.avg_price).await?;
```

## 🧪 Testes

```bash
# Unit tests
cargo test --workspace

# Específico do market-intelligence
cargo test -p arcsat-market-intelligence

# Com logs
RUST_LOG=debug cargo test --workspace -- --nocapture
```

## 🔒 Segurança

1. **Multi-tenancy**: Todos os dados são isolados por tenant
2. **API Auth**: JWT obrigatório (TODO: implementar middleware)
3. **Rate Limiting**: Redis para controle de requisições
4. **GDPR/LGPD**: Dados anônimos, sem PII

## 📈 Performance

### Métricas Esperadas

- **Scraping**: 50-100 produtos/minuto (sem proxy)
- **Scraping com proxy**: 200-500 produtos/minuto
- **Latência API**: <50ms (P95)
- **Throughput**: 1000+ req/s

### Otimizações

1. **Paralelização**: Múltiplos workers processando filas
2. **Caching**: Redis para resultados recentes
3. **Batch Processing**: Processar múltiplas páginas em paralelo

## 🚧 Roadmap

### Curto Prazo (1-2 semanas)
- [ ] Implementar autenticação JWT
- [ ] Persistência em PostgreSQL (migrations)
- [ ] Worker separado para processar filas
- [ ] Webhooks para notificação de jobs completados

### Médio Prazo (1 mês)
- [ ] Dashboard web para visualização
- [ ] Scrapers para B2W, Magalu, Shopee
- [ ] ML para detecção de oportunidades
- [ ] Alertas automáticos (Telegram/Email)

### Longo Prazo (2-3 meses)
- [ ] IA para análise semântica de produtos
- [ ] Previsão de demanda
- [ ] Integração com módulos ERP/CRM
- [ ] API pública para parceiros

## 💰 Custos Estimados

### Desenvolvimento
- **VPS**: $5-20/mês (Hetzner, DigitalOcean)
- **PostgreSQL**: $0 (self-hosted) ou $15/mês (managed)
- **Redis**: $0 (self-hosted) ou $10/mês (Redis Cloud)

### Produção (scraping intenso)
- **Proxies (Bright Data)**: $300-500/mês
- **VPS (4-8 cores)**: $40-80/mês
- **Total**: ~$400-650/mês

### Alternativa Barata
- **ScraperAPI**: $49/mês (5000 requests)
- **VPS básica**: $5/mês
- **Total**: ~$60/mês (limitado)

## 🤝 Contribuindo

Este é um módulo do Arcsat ERP. Para contribuir:

1. Fork o repositório
2. Crie uma branch (`git checkout -b feature/nova-feature`)
3. Commit (`git commit -am 'Add nova feature'`)
4. Push (`git push origin feature/nova-feature`)
5. Abra um Pull Request

## 📝 Licença

Proprietário - Arcsat/AvilaOps © 2025

---

**Desenvolvido com ❤️ usando Rust e as ferramentas do Arxis-Core**
