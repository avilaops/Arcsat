# 🚨 Railway 404 - Guia Rápido de Fix

## Status Atual
- ❌ djangoapp-production-62bd.up.railway.app → 404
- ❌ core-production-028a.up.railway.app → 404
- ❌ backend-production-5b7d.up.railway.app → 404

## 🔍 Diagnóstico

Execute:
```powershell
.\check-railway-services.ps1
```

## 🛠️ Fix Rápido

### 1. Verificar Logs no Railway

```bash
# Instalar Railway CLI (se não tiver)
npm install -g @railway/cli

# Login
railway login

# Link ao projeto
railway link

# Ver logs de cada serviço
railway logs --service backend
railway logs --service core
railway logs --service djangoapp
```

### 2. Verificar Variáveis de Ambiente

No Railway Dashboard, cada serviço precisa de:

**Backend (Rust)**:
```
PORT=3000
RUST_LOG=info
DATABASE_URL=postgresql://...  (auto se tiver PostgreSQL)
REDIS_URL=redis://...  (auto se tiver Redis)
MI_PROXY_ENABLED=false
```

**Core (Django)**:
```
PORT=8001
DJANGO_SETTINGS_MODULE=core.settings
SECRET_KEY=...
DATABASE_URL=postgresql://...
```

**Django App (Frontend)**:
```
PORT=8000
API_URL=https://backend-production-5b7d.up.railway.app
```

### 3. Re-Deploy

```powershell
# Método 1: CLI
.\deploy-railway.ps1

# Método 2: Git push
git add .
git commit -m "fix: railway deployment configuration"
git push origin master

# Railway auto-deploys se conectado ao GitHub
```

### 4. Adicionar Databases

No Railway Dashboard:
1. Click **"New"** → **"Database"** → **"PostgreSQL"**
2. Click **"New"** → **"Database"** → **"Redis"**
3. Em cada serviço, adicionar referências aos bancos

## 🎯 Checklist de Troubleshooting

- [ ] Railway CLI instalado e autenticado
- [ ] Projeto linkado (`railway link`)
- [ ] Logs verificados (erros de build/runtime)
- [ ] Dockerfile correto (binary name = arcsat-server)
- [ ] railway.json com startCommand correto
- [ ] PORT configurado em env vars
- [ ] PostgreSQL e Redis provisionados
- [ ] Health check endpoint funcionando (`/api/v1/health`)

## 📊 Estrutura de Services

```
Arcsat Project (Railway)
├── backend (Rust)
│   ├── Dockerfile: Dockerfile.railway
│   ├── Start: /app/arcsat-server
│   └── Port: 3000
├── core (Django)
│   ├── Start: gunicorn core.wsgi
│   └── Port: 8001
├── djangoapp (Frontend)
│   ├── Start: python manage.py runserver
│   └── Port: 8000
├── PostgreSQL
│   └── Port: 5432 (internal)
└── Redis
    └── Port: 6379 (internal)
```

## ⚡ Ação Imediata

```powershell
# 1. Verificar status
railway status

# 2. Ver logs de cada serviço
railway logs --service backend
railway logs --service core
railway logs --service djangoapp

# 3. Se build falhou, re-deploy:
railway up

# 4. Testar após deploy:
.\check-railway-services.ps1
```

## 🆘 Se ainda não funcionar

**Causa provável**: Build está falhando

**Solução**:
1. No Railway Dashboard, ir em cada serviço
2. Click em **"Deployments"**
3. Ver logs do último deployment
4. Procurar por:
   - ❌ `build failed`
   - ❌ `Error: ENOENT`
   - ❌ `panic at`
   - ❌ `connection refused`

**Erros comuns**:
- `arcsat-server: not found` → Binary name errado no Dockerfile
- `failed to connect to postgres` → DATABASE_URL não configurado
- `Address already in use` → PORT conflitando

## 📚 Documentação Completa

Ver: [RAILWAY_SERVICES.md](RAILWAY_SERVICES.md)

---

**Próximo passo**: Execute `railway logs --service backend` e me envie o output!
