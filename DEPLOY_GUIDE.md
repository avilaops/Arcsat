# 🚀 Deploy Arcsat no Railway - Sem CLI

## Problema Atual
Railway CLI não instalado. Vamos usar **GitHub Integration** (mais simples!).

---

## ✅ Método 1: GitHub Auto-Deploy (RECOMENDADO)

### Passo 1: Prepare o Código

```powershell
# 1. Commit das mudanças
git add .
git commit -m "feat: complete market intelligence + CRM integration"
git push origin master
```

### Passo 2: Conecte ao Railway

1. Acesse: https://railway.app/dashboard
2. Click **"New Project"**
3. Selecione **"Deploy from GitHub repo"**
4. Escolha: **avilaops/Arcsat**
5. Railway vai detectar automaticamente:
   - ✅ `Dockerfile.railway` presente
   - ✅ `railway.json` com configuração

### Passo 3: Configure Variáveis de Ambiente

No Railway Dashboard, adicione:

```bash
PORT=3000
RUST_LOG=info
MI_PROXY_ENABLED=false
MI_MAX_CONCURRENT_JOBS=5
```

### Passo 4: Adicione Databases

1. Click **"New"** → **"Database"** → **"PostgreSQL"**
   - Railway cria automaticamente e injeta `DATABASE_URL`

2. Click **"New"** → **"Database"** → **"Redis"**
   - Railway cria automaticamente e injeta `REDIS_URL`

### Passo 5: Deploy Automático

Railway detecta push no GitHub e faz deploy automaticamente! 🎉

**Tempo estimado**: 10-15 minutos (primeira compilação do Rust)

---

## ✅ Método 2: Railway CLI (se quiser instalar)

### Instalar Railway CLI

**Node.js/npm** (Recomendado):
```powershell
npm install -g @railway/cli
```

**Windows (Standalone)**:
```powershell
# PowerShell como Admin
iwr https://railway.app/install.ps1 | iex
```

**Scoop**:
```powershell
scoop install railway
```

### Usar CLI

```powershell
# Login
railway login

# Link ao projeto
railway link

# Deploy
railway up

# Ver logs
railway logs
```

---

## ✅ Método 3: Docker Build Local + Push Manual

Se quiser testar antes:

```powershell
# Build local
docker build -f Dockerfile.railway -t arcsat-backend .

# Run local
docker run -p 3000:3000 `
  -e PORT=3000 `
  -e RUST_LOG=info `
  arcsat-backend

# Testar
curl http://localhost:3000/api/v1/health
```

---

## 🎯 Configuração Completa do Railway Project

### Services a criar:

```
📦 Arcsat (Railway Project)
│
├── 🦀 backend (Rust API)
│   ├── Source: GitHub avilaops/Arcsat
│   ├── Root: /
│   ├── Dockerfile: Dockerfile.railway
│   ├── Env:
│   │   PORT=3000
│   │   RUST_LOG=info
│   │   DATABASE_URL=${{Postgres.DATABASE_URL}}
│   │   REDIS_URL=${{Redis.REDIS_URL}}
│   │   MI_PROXY_ENABLED=false
│   └── Domain: backend-production-5b7d.up.railway.app
│
├── 🗄️ PostgreSQL
│   └── Auto-generated DATABASE_URL
│
└── 📮 Redis
    └── Auto-generated REDIS_URL
```

### Para Core e Django App:

Se você tem esses serviços separados, crie services adicionais:

**Core (Django)**:
```
Source: Mesmo repo ou outro
Root: /core  (se for subdiretório)
Start Command: gunicorn core.wsgi:application --bind 0.0.0.0:$PORT
Env:
  PORT=8001
  DJANGO_SETTINGS_MODULE=core.settings
  DATABASE_URL=${{Postgres.DATABASE_URL}}
```

**Django App (Frontend)**:
```
Source: Mesmo repo
Root: /frontend (se aplicável)
Start Command: python manage.py runserver 0.0.0.0:$PORT
Env:
  PORT=8000
  BACKEND_API_URL=https://backend-production-5b7d.up.railway.app
```

---

## 📊 Verificar Deploy

Após deploy completo (10-15 min):

```powershell
# Script de verificação
.\check-railway-services.ps1

# Ou manualmente:
curl https://backend-production-5b7d.up.railway.app/api/v1/health
```

**Resposta esperada**:
```json
{
  "status": "ok",
  "version": "1.0.0",
  "timestamp": "2025-12-30T..."
}
```

---

## 🐛 Troubleshooting

### Build Timeout

**Problema**: Rust compilation > 10 min

**Solução**: No Railway Dashboard
1. Settings → Build
2. Aumentar timeout para 20 minutos
3. Ou usar cache:

```dockerfile
# No Dockerfile.railway, adicionar antes do build:
ENV CARGO_HOME=/cargo
ENV CARGO_TARGET_DIR=/cargo/target
VOLUME /cargo
```

### 404 Ainda Aparece

**Causa**: Binary não inicia ou PORT errado

**Fix**:
1. Railway Dashboard → Deployments → Ver logs
2. Procurar por:
   ```
   Error: Address already in use
   panic at 'DATABASE_URL must be set'
   ```
3. Corrigir env vars ou Dockerfile

### Out of Memory

**Causa**: Rust build usa muita RAM (1GB+)

**Solução**: Upgrade Railway plan
- Hobby: $5/mês, 512MB → 1GB
- Pro: $20/mês, 8GB

---

## 🎁 Bonus: Nixpacks (Alternativa ao Dockerfile)

Railway suporta Nixpacks (auto-detect):

```toml
# nixpacks.toml
[phases.setup]
nixPkgs = ["chromium", "postgresql"]

[phases.build]
cmds = ["cargo build --release"]

[start]
cmd = "./target/release/arcsat-server"
```

---

## ✅ Checklist Final

- [ ] Código commitado e pushed para GitHub
- [ ] Railway Project criado
- [ ] Repositório GitHub conectado
- [ ] PostgreSQL database adicionado
- [ ] Redis database adicionado
- [ ] Variáveis de ambiente configuradas
- [ ] Deploy automático ativado (GitHub push)
- [ ] Aguardar 10-15 min (primeira compilação)
- [ ] Testar endpoints com `check-railway-services.ps1`
- [ ] Dashboard em: https://backend-production-5b7d.up.railway.app/

---

## 🚀 Deploy Agora!

```powershell
# 1. Push para GitHub
git add .
git commit -m "feat: arcsat backend with market intelligence"
git push origin master

# 2. Configurar no Railway Dashboard:
# https://railway.app/new

# 3. Aguardar deploy automático

# 4. Verificar
.\check-railway-services.ps1
```

**Tempo total**: ~15 minutos

**URL final**: https://backend-production-5b7d.up.railway.app/api/v1/health

---

**Dúvidas?** Me envie os logs do Railway Dashboard (Deployments tab)!
