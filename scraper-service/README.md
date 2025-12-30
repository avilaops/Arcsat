# Arcsat Market Intelligence - Guia de Integração

## 🎯 O que foi criado

Um microserviço completo de **Market Intelligence** com:
- ✅ Web scraping resiliente (Playwright + stealth)
- ✅ Sistema de filas (Redis)
- ✅ API REST (FastAPI)
- ✅ Suporte a múltiplos marketplaces (Amazon, Mercado Livre)
- ✅ Anti-detecção (fingerprinting, proxies rotativos)
- ✅ Docker/docker-compose pronto

## 🏗️ Arquitetura

```
┌─────────────────┐
│  Frontend React │
└────────┬────────┘
         │ HTTP
┌────────▼────────┐      ┌──────────────┐
│  Backend Rust   │◄────►│  PostgreSQL  │
│   (Axum/3000)   │      └──────────────┘
└────────┬────────┘
         │ HTTP
┌────────▼────────┐      ┌──────────────┐
│ Scraper Service │◄────►│    Redis     │
│  (Python/8001)  │      │ (Queue/Cache)│
└────────┬────────┘      └──────────────┘
         │
    ┌────▼─────┐
    │ Proxies  │
    │(opcional)│
    └──────────┘
```

## 🚀 Como rodar localmente

### 1. Instalar dependências do Scraper

```bash
cd scraper-service
python -m venv venv
.\venv\Scripts\activate  # Windows
# source venv/bin/activate  # Linux/Mac
pip install -r requirements.txt
playwright install chromium
```

### 2. Configurar .env

```bash
cp .env.example .env
# Editar .env com suas configurações
```

### 3. Rodar com Docker Compose

```bash
# Na raiz do projeto
docker-compose up -d
```

Isso vai subir:
- Backend Rust (porta 3000)
- Scraper Service (porta 8001)
- PostgreSQL (porta 5432)
- Redis (porta 6379)
- Redis Commander UI (porta 8081)

### 4. Testar o Scraper

```bash
# Criar um job de scraping
curl -X POST http://localhost:8001/api/v1/scraping/jobs \
  -H "Content-Type: application/json" \
  -d '{
    "marketplace": "mercado_livre",
    "search_query": "notebook gamer",
    "max_pages": 3,
    "priority": 5
  }'

# Resposta:
# {
#   "job_id": "uuid-aqui",
#   "status": "pending",
#   "message": "Job criado e adicionado à fila"
# }

# Verificar status
curl http://localhost:8001/api/v1/scraping/jobs/{job_id}
```

## 🔌 Integração com o Backend Rust

### Adicionar no seu backend Rust (Axum):

```rust
// Adicionar no Cargo.toml
[dependencies]
reqwest = { version = "0.11", features = ["json"] }

// No seu código Rust
use reqwest::Client;
use serde::{Deserialize, Serialize};

#[derive(Serialize)]
struct ScrapingJobRequest {
    marketplace: String,
    search_query: String,
    max_pages: i32,
    priority: i32,
}

#[derive(Deserialize)]
struct ScrapingJobResponse {
    job_id: String,
    status: String,
    message: String,
}

// Endpoint no Rust que chama o Scraper
async fn create_market_analysis(
    Query(params): Query<MarketAnalysisParams>,
) -> Result<Json<ScrapingJobResponse>, StatusCode> {
    let client = Client::new();

    let job = ScrapingJobRequest {
        marketplace: params.marketplace,
        search_query: params.query,
        max_pages: 5,
        priority: 5,
    };

    let scraper_url = std::env::var("SCRAPER_SERVICE_URL")
        .unwrap_or_else(|_| "http://localhost:8001".to_string());

    let response = client
        .post(format!("{}/api/v1/scraping/jobs", scraper_url))
        .json(&job)
        .send()
        .await
        .map_err(|_| StatusCode::SERVICE_UNAVAILABLE)?;

    let result: ScrapingJobResponse = response
        .json()
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok(Json(result))
}
```

## 📊 Próximos Passos (TODOs)

### Curto Prazo (1-2 semanas)
- [ ] Implementar persistência no PostgreSQL (models SQLAlchemy)
- [ ] Criar tabelas de migração (Alembic)
- [ ] Adicionar análise de tendências (pandas/numpy)
- [ ] Implementar sistema de keywords (NLP básico)
- [ ] Criar endpoints no backend Rust

### Médio Prazo (1 mês)
- [ ] Integrar proxies rotativos (Bright Data/Oxylabs)
- [ ] Implementar resolução de CAPTCHA (2captcha/AntiCaptcha)
- [ ] Criar dashboard no frontend (gráficos com Chart.js)
- [ ] Sistema de alertas (produtos em alta)
- [ ] API de histórico de preços

### Longo Prazo (2-3 meses)
- [ ] ML para previsão de tendências
- [ ] Scraping de mais marketplaces (Shopee, B2W, Magalu)
- [ ] Sistema de recomendação de nichos
- [ ] Integração com IA (GPT) para análise semântica
- [ ] Exportação de relatórios (PDF/Excel)

## 💰 Custos Estimados

### Sem proxies (apenas testes locais)
- **Custo:** $0/mês
- **Limitação:** Alto risco de bloqueio, 10-50 requisições/dia

### Com proxies residenciais (produção)
- **Bright Data:** ~$500/mês (40GB de dados)
- **Oxylabs:** ~$300/mês (starter plan)
- **Alternativa:** Scraperapi ($49-99/mês, mas menos controle)

### Infraestrutura
- **VPS básica (Hetzner):** €4.5/mês (2vCPU, 4GB RAM)
- **Railway/Fly.io:** ~$20-50/mês
- **AWS/GCP:** ~$50-100/mês (com auto-scaling)

## ⚠️ Avisos Legais

1. **Termos de Serviço:** Scraping viola os ToS da maioria dos marketplaces
2. **Uso Responsável:**
   - Rate limiting adequado
   - Respeitar robots.txt (ou não, sua escolha)
   - Não causar sobrecarga nos servidores
3. **Risco Legal:**
   - Baixo para usuário final
   - Médio-Alto para quem opera o serviço
   - Amazon já processou ferramentas similares

## 🛠️ Troubleshooting

### Erro: "Playwright browser not found"
```bash
playwright install chromium
```

### Redis connection refused
```bash
# Verificar se Redis está rodando
docker ps | grep redis
# Ou instalar localmente
# Windows: https://github.com/microsoftarchive/redis/releases
```

### Scraping muito lento
- Reduza `max_pages`
- Aumente `SCRAPER_MAX_CONCURRENT`
- Verifique se proxy está ativo

### CAPTCHAs constantes
- Ative proxies residenciais
- Reduza frequência de requisições
- Implemente 2captcha/AntiCaptcha

## 📚 Documentação API

Acesse: http://localhost:8001/docs (Swagger UI automático do FastAPI)

## 🤝 Contribuindo

Este é um sistema complexo. Sugestões de melhorias:
- Implementar Celery para filas robustas
- Adicionar testes (pytest)
- Monitoramento (Prometheus + Grafana)
- Logs estruturados (structlog)
- Circuit breaker para resiliência

---

**Verdade nua e crua:**
- ✅ Tecnicamente viável
- ⚠️ Legalmente questionável
- 💰 Operacionalmente caro (se escalar)
- 🎯 Comercialmente valioso (se funcionar)
- ⏰ Manutenção contínua necessária (sites mudam)

Boa sorte! 🚀
