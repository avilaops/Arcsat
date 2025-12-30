# 🚂 Railway Deploy - Guia Completo

## Status Atual
✅ Deploy preparado
⚠️ Serviços retornando 404 - Correções aplicadas

## 🔧 Correções Aplicadas

### 1. railway.json
- ✅ Healthcheck path corrigido: `/health`
- ✅ Dockerfile.railway configurado corretamente
- ✅ Start command: `/app/arcsat-server`

### 2. main.rs
- ✅ Endpoint `/api/v1/health` adicionado
- ✅ Variável `PORT` do Railway configurada
- ✅ Health check robusto implementado

### 3. Dockerfile.railway
- ✅ Rust 1.86 (suporta edition2024)
- ✅ Chrome para scraping
- ✅ Binary correto: `arcsat-server`

---

## 📋 Checklist de Deploy

### Pré-Deploy Local
```powershell
# 1. Build local para testar
.\build-backend.ps1

# 2. Testar localmente
cd arcsat-backend
cargo run --release --bin arcsat-server

# 3. Verificar endpoints
curl http://localhost:3000/health
curl http://localhost:3000/api/v1/health
```

### Deploy no Railway

#### Opção 1: Via GitHub (Recomendado)

```powershell
# 1. Commit as mudanças
git add .
git commit -m "fix: railway deploy configuration"
git push origin master
```

2. No Railway Dashboard:
   - O deploy automático vai iniciar
   - Aguarde build completar (~5-10 min)
   - Verifique logs em real-time

#### Opção 2: Via Railway CLI

```powershell
# Instalar Railway CLI
npm install -g @railway/cli

# Login
railway login

# Deploy
railway up
```

---

## ⚙️ Variáveis de Ambiente Necessárias

Configure no Railway Dashboard → Settings → Variables:

### Backend (Rust)
```bash
PORT=3000
RUST_LOG=info
DATABASE_URL=postgresql://user:pass@host/db
REDIS_URL=redis://host:6379

# Market Intelligence
MI_PROXY_ENABLED=false
MI_MAX_CONCURRENT_JOBS=5
MI_JOB_TIMEOUT=300

# Bright Data (se usar proxy)
BRIGHTDATA_USERNAME=your_username
BRIGHTDATA_PASSWORD=your_password
BRIGHTDATA_HOST=brd.superproxy.io
BRIGHTDATA_PORT=22225
```

### Core/Django (se houver)
```bash
PORT=8000
DJANGO_SECRET_KEY=your_secret_key
DATABASE_URL=postgresql://...
ALLOWED_HOSTS=core-production-028a.up.railway.app
```

---

## 🔍 Verificação Pós-Deploy

### 1. Check Health
```powershell
# Executar script de verificação
.\check-railway-services.ps1

# Ou manualmente
curl https://backend-production-5b7d.up.railway.app/health
curl https://backend-production-5b7d.up.railway.app/api/v1/health
```

### 2. Test Market Intelligence API
```powershell
# Criar job
$body = @{
    marketplace = "amazon"
    search_query = "smartphone"
    max_pages = 2
    priority = 5
} | ConvertTo-Json

Invoke-RestMethod -Method Post `
    -Uri "https://backend-production-5b7d.up.railway.app/api/v1/market-intelligence/jobs" `
    -ContentType "application/json" `
    -Body $body
```

### 3. Check Logs
No Railway Dashboard:
- View → Logs
- Filtrar por service: backend
- Procurar por:
  - ✅ `Starting Arcsat API Server`
  - ✅ `Server listening on 0.0.0.0:3000`
  - ❌ Erros de bind ou startup

---

## 🐛 Troubleshooting

### Erro: "Application not found"
**Causa**: Build falhou ou service não deployou
**Solução**:
1. Check Railway logs
2. Verificar se Dockerfile.railway está correto
3. Re-trigger deploy: `railway up --detach`

### Erro: "Health check failed"
**Causa**: App não está respondendo na PORT correta
**Solução**:
1. Verificar variável `PORT` está configurada
2. Logs devem mostrar: `Server listening on 0.0.0.0:3000`
3. Ajustar healthcheck timeout se necessário

### Build Timeout
**Causa**: Rust demora muito para compilar
**Solução**:
1. Usar cache: Railway → Settings → Enable Build Cache
2. Considerar usar binary pré-compilado
3. Otimizar dependencies no Cargo.toml

### Database Connection Error
**Causa**: DATABASE_URL não configurado
**Solução**:
1. Railway → Add → Database → PostgreSQL
2. Link service ao database
3. DATABASE_URL é injetado automaticamente

---

## 📊 Monitoramento

### Métricas no Railway
- CPU usage
- Memory usage
- Request count
- Response time

### Logs em Tempo Real
```bash
railway logs --follow
```

### Alertas (Railway Pro)
Configure no Dashboard:
- Health check failures
- High memory usage
- Deployment failures

---

## 🚀 Próximos Passos

1. **Custom Domain**
   ```
   Railway → Settings → Domains → Add Custom Domain
   ```

2. **Environment Segregation**
   - Criar environment `staging`
   - Testar antes de production

3. **CI/CD Automation**
   - GitHub Actions para testes
   - Deploy automático em merge

4. **Monitoring**
   - Integrar Sentry para errors
   - Application Insights para métricas

5. **Scaling**
   - Configurar auto-scaling
   - Otimizar resource limits

---

## 💰 Custos Estimados

### Starter Plan ($5/mês)
- 512 MB RAM por service
- 1 GB storage
- $0.000463/GB egress

### Com Add-ons
- PostgreSQL: $5/mês (1GB)
- Redis: $5/mês (256MB)

**Total**: ~$15-20/mês para 1 service + databases

---

## 📚 Referências

- [Railway Docs](https://docs.railway.app)
- [Rust Railway Template](https://github.com/railway/templates/tree/main/rust)
- [Dockerfile Best Practices](https://docs.docker.com/develop/dev-best-practices/)
