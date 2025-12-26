# 🏢 ERP/CRM Faria Lima - Backend

Software ERP/CRM enterprise em Rust para o mercado corporativo premium.

## 🚀 Features Implementadas

### ✅ Core
- **Autenticação Enterprise**: JWT, SSO (Google), MFA (TOTP)
- **Multi-tenancy**: Isolamento completo por tenant (subdomain ou header)
- **RBAC**: Sistema granular de permissões (crm.read, finance.write, etc)
- **Webhooks**: Notificações em tempo real com HMAC SHA-256

### ✅ Módulos

#### CRM
- CRUD completo de Leads
- Pipeline visual (Kanban)
- Lead Scoring com IA
- Forecast de vendas
- Health Score de contas (360° view)
- Histórico de mudanças
- Accounts, Contacts, Activities

#### Financeiro
- Contas a Pagar/Receber
- Projeção de Cashflow (12-18 meses)
- DRE em tempo real
- Pagamentos PIX com QR Code
- Emissão de NFe (estrutura pronta)
- Conciliação bancária

#### RH
- CRUD de funcionários
- Folha de pagamento (cálculo INSS, IRRF, encargos)
- Ponto eletrônico
- Avaliações de desempenho
- Analytics (turnover, headcount)

## 📋 Pré-requisitos

- Rust 1.75+
- PostgreSQL 15+
- Redis 7+ (opcional, para cache)
- Docker (opcional)

## 🔧 Configuração

### 1. Clonar repositório

```bash
git clone https://github.com/avilaops/ERP.git
cd ERP/backend
```

### 2. Configurar variáveis de ambiente

```bash
cp .env.example .env
```

Editar `.env`:

```env
# Database
DATABASE_URL=postgresql://user:password@localhost:5432/erp

# Auth
JWT_SECRET=your_super_secret_key_min_32_chars_long_here

# Server
HOST=0.0.0.0
PORT=3000
RUN_MIGRATIONS=true

# OAuth (opcional)
GOOGLE_CLIENT_ID=your_google_client_id
GOOGLE_CLIENT_SECRET=your_google_secret

# Redis (opcional)
REDIS_URL=redis://localhost:6379
```

### 3. Criar banco de dados

```bash
# PostgreSQL
createdb erp

# Ou via Docker
docker run -d \
  --name erp-postgres \
  -e POSTGRES_DB=erp \
  -e POSTGRES_USER=erp_user \
  -e POSTGRES_PASSWORD=secure_password \
  -p 5432:5432 \
  postgres:15
```

### 4. Executar migrations

```bash
# Instalar sqlx-cli
cargo install sqlx-cli --no-default-features --features postgres

# Rodar migrations
sqlx migrate run
```

Ou use o SQL completo:

```bash
psql -U postgres -d erp -f ../database_schema.sql
```

### 5. Compilar e rodar

```bash
# Desenvolvimento (com hot-reload)
cargo watch -x run

# Ou compilar e rodar
cargo build
cargo run

# Produção
cargo build --release
./target/release/erp-backend
```

## 📊 Estrutura do Projeto

```
backend/
├── src/
│   ├── main.rs              # Servidor principal
│   ├── lib.rs               # Biblioteca
│   ├── auth.rs              # Autenticação JWT/SSO/MFA
│   ├── db.rs                # Connection pooling
│   ├── error.rs             # Sistema de erros
│   ├── models.rs            # Modelos de dados
│   ├── webhooks.rs          # Sistema de webhooks
│   ├── routes/
│   │   ├── crm.rs           # APIs CRM
│   │   ├── finance.rs       # APIs Financeiro
│   │   └── hr.rs            # APIs RH
│   └── middleware/
│       ├── tenant.rs        # Multi-tenancy
│       └── rbac.rs          # Permissões
├── migrations/              # SQL migrations
├── Cargo.toml
└── .env
```

## 🧪 Testes

```bash
# Rodar todos os testes
cargo test

# Testes com output detalhado
cargo test -- --nocapture

# Teste específico
cargo test test_calculate_lead_score

# Coverage
cargo tarpaulin --out Html
```

## 📖 Documentação da API

### Autenticação

#### Login
```bash
curl -X POST http://localhost:3000/api/v1/auth/login \
  -H "Content-Type: application/json" \
  -d '{
    "email": "usuario@empresa.com",
    "password": "senha_segura",
    "tenant_domain": "empresa"
  }'
```

Response:
```json
{
  "access_token": "eyJhbGciOiJIUzI1NiIs...",
  "refresh_token": "dGhpc2lzYXJlZnJlc2h...",
  "expires_in": 28800,
  "user": {
    "id": "uuid",
    "name": "João Silva",
    "email": "usuario@empresa.com",
    "roles": ["admin"],
    "tenant_id": "tenant-uuid"
  }
}
```

### CRM - Leads

#### Criar Lead
```bash
curl -X POST http://localhost:3000/api/v1/crm/leads \
  -H "Authorization: Bearer {token}" \
  -H "Content-Type: application/json" \
  -d '{
    "name": "Maria Oliveira",
    "company": "Tech Corp",
    "email": "maria@techcorp.com",
    "phone": "+5511999887766",
    "source": "linkedin",
    "value": 150000.00
  }'
```

#### Listar Leads
```bash
curl http://localhost:3000/api/v1/crm/leads?page=1&limit=50 \
  -H "Authorization: Bearer {token}"
```

#### Pipeline Visual
```bash
curl http://localhost:3000/api/v1/crm/opportunities/pipeline \
  -H "Authorization: Bearer {token}"
```

#### Forecast
```bash
curl http://localhost:3000/api/v1/crm/opportunities/forecast \
  -H "Authorization: Bearer {token}"
```

### Financeiro

#### Projeção de Cashflow
```bash
curl -X POST http://localhost:3000/api/v1/finance/cashflow/projection \
  -H "Authorization: Bearer {token}" \
  -H "Content-Type: application/json" \
  -d '{
    "start_date": "2024-02-01",
    "months": 12,
    "scenario": "realistic"
  }'
```

#### DRE em Tempo Real
```bash
curl http://localhost:3000/api/v1/finance/dre/realtime \
  -H "Authorization: Bearer {token}"
```

### RH

#### Calcular Folha de Pagamento
```bash
curl -X POST http://localhost:3000/api/v1/hr/payroll/calculate \
  -H "Authorization: Bearer {token}" \
  -H "Content-Type: application/json" \
  -d '{
    "reference_month": "2024-01",
    "include_benefits": true,
    "include_overtime": true
  }'
```

## 🔐 Segurança

### Roles e Permissões

**Admin**: Acesso total
```rust
permissions: ["*"]
```

**Sales Manager**: Gerenciar CRM
```rust
permissions: ["crm.*", "analytics.read"]
```

**Sales Rep**: CRM básico
```rust
permissions: ["crm.read", "crm.write", "leads.*"]
```

**Finance Manager**: Gestão financeira
```rust
permissions: ["finance.*", "reports.read"]
```

**HR Manager**: Gestão de RH
```rust
permissions: ["hr.*", "employees.*", "payroll.*"]
```

### Multi-tenancy

Extrair tenant por:

1. **Subdomain** (recomendado em produção):
```
empresa.erp.com -> tenant_id lookup
```

2. **Header customizado** (útil para APIs):
```
X-Tenant-ID: uuid-do-tenant
```

## 📈 Performance

### Benchmarks Esperados

- **API Response**: < 200ms (P95)
- **Database Queries**: < 50ms
- **Concurrent Users**: 10.000+
- **Throughput**: 5.000 req/s

### Otimizações Aplicadas

- Connection pooling (100 conexões)
- Prepared statements
- Índices otimizados
- Async/await nativo
- Zero-copy serialization

## 🐳 Docker

```dockerfile
# Dockerfile
FROM rust:1.75 as builder
WORKDIR /app
COPY . .
RUN cargo build --release

FROM debian:bookworm-slim
RUN apt-get update && apt-get install -y libpq5 ca-certificates
COPY --from=builder /app/target/release/erp-backend /usr/local/bin/
CMD ["erp-backend"]
```

```bash
# Build
docker build -t erp-backend .

# Run
docker run -p 3000:3000 --env-file .env erp-backend
```

### Docker Compose

```yaml
version: '3.8'
services:
  postgres:
    image: postgres:15
    environment:
      POSTGRES_DB: erp
      POSTGRES_USER: erp_user
      POSTGRES_PASSWORD: secure_password
    ports:
      - "5432:5432"
    volumes:
      - postgres_data:/var/lib/postgresql/data

  redis:
    image: redis:7-alpine
    ports:
      - "6379:6379"

  backend:
    build: .
    ports:
      - "3000:3000"
    depends_on:
      - postgres
      - redis
    environment:
      DATABASE_URL: postgresql://erp_user:secure_password@postgres:5432/erp
      REDIS_URL: redis://redis:6379
      JWT_SECRET: your_secret_key

volumes:
  postgres_data:
```

```bash
docker-compose up -d
```

## 🚀 Deploy

### Railway / Render / Fly.io

1. Configure variáveis de ambiente
2. Conecte repositório GitHub
3. Deploy automático no push

### VPS / Bare Metal

```bash
# Build release
cargo build --release

# Copiar binário
scp target/release/erp-backend user@server:/opt/erp/

# Systemd service
sudo nano /etc/systemd/system/erp-backend.service
```

```ini
[Unit]
Description=ERP/CRM Backend
After=network.target postgresql.service

[Service]
Type=simple
User=erp
WorkingDirectory=/opt/erp
EnvironmentFile=/opt/erp/.env
ExecStart=/opt/erp/erp-backend
Restart=on-failure

[Install]
WantedBy=multi-user.target
```

```bash
sudo systemctl enable erp-backend
sudo systemctl start erp-backend
sudo systemctl status erp-backend
```

## 📝 Próximos Passos

### MVP (2 semanas)
- [x] Implementar activities completas
- [ ] Adicionar contacts completos
- [ ] Sistema de cache Redis
- [ ] Rate limiting por tenant
- [ ] Testes de integração

### V1.0 (1 mês)
- [ ] Analytics avançado
- [ ] Integrações (WhatsApp, LinkedIn)
- [ ] ML para lead scoring real
- [ ] Mobile API otimizada
- [ ] Multi-idioma (i18n)

### V2.0 (3 meses)
- [ ] Workflow engine visual
- [ ] Custom fields ilimitados
- [ ] Marketplace de integrações
- [ ] IA generativa (GPT-4)
- [ ] Compliance SOC2

## 🤝 Contribuindo

1. Fork o projeto
2. Crie uma branch (`git checkout -b feature/nova-feature`)
3. Commit suas mudanças (`git commit -m 'Add: nova feature'`)
4. Push para a branch (`git push origin feature/nova-feature`)
5. Abra um Pull Request

## 📄 Licença

MIT OR Apache-2.0

## 👥 Autores

- **Nícolas Ávila** - [nicolas@avila.inc](mailto:nicolas@avila.inc)

## 🙏 Agradecimentos

- Rust community
- Axum framework
- SQLx team
- Tokio runtime

---

**Made with ❤️ and 🦀 Rust for Faria Lima**
