# Railway Multi-Service Architecture

## 🚂 Deployed Services

### 1. **Backend** (Rust API)
**URL**: https://backend-production-5b7d.up.railway.app
- Main Arcsat API server
- Market Intelligence endpoints
- CRM integration
- Built from: `arcsat-backend/`

### 2. **Core** (Django API)
**URL**: https://core-production-028a.up.railway.app
- Legacy Python/Django core
- ERP modules
- Authentication

### 3. **Django App** (Frontend)
**URL**: https://djangoapp-production-62bd.up.railway.app
- Web interface
- Static assets
- Dashboard

---

## 🔧 Configuration

### Backend Service (Rust)

**railway.json**:
```json
{
  "$schema": "https://railway.app/railway.schema.json",
  "build": {
    "builder": "DOCKERFILE",
    "dockerfilePath": "Dockerfile.railway"
  },
  "deploy": {
    "startCommand": "/app/arcsat-server",
    "restartPolicyType": "ON_FAILURE",
    "restartPolicyMaxRetries": 10
  }
}
```

**Environment Variables**:
```bash
PORT=3000
RUST_LOG=info
DATABASE_URL=postgresql://user:pass@postgres.railway.internal:5432/arcsat
REDIS_URL=redis://redis.railway.internal:6379
MI_PROXY_ENABLED=false
MI_MAX_CONCURRENT_JOBS=5
```

### Required Services

Add these services in Railway dashboard:

1. **PostgreSQL**
   - Click "New" → "Database" → "PostgreSQL"
   - Railway provides `DATABASE_URL` automatically

2. **Redis**
   - Click "New" → "Database" → "Redis"
   - Railway provides `REDIS_URL` automatically

---

## 🚀 Deployment Workflow

### Option 1: Railway CLI
```bash
# Install Railway CLI
npm i -g @railway/cli

# Login
railway login

# Link project
railway link

# Deploy backend
railway up
```

### Option 2: GitHub Integration
1. Connect repository to Railway
2. Select service to deploy
3. Set root directory to `/` for backend
4. Railway auto-deploys on push to `master`

---

## 🐛 Troubleshooting

### Issue: 404 on all endpoints

**Possible causes**:
1. ❌ Build failed - Check Railway logs
2. ❌ Start command incorrect in railway.json
3. ❌ Binary name mismatch (arcsat-server vs backend)
4. ❌ PORT not bound correctly

**Fix**:
```dockerfile
# In Dockerfile.railway, ensure correct binary name:
COPY --from=builder /build/arcsat-backend/target/release/arcsat-server /app/arcsat-server

# In railway.json:
"startCommand": "/app/arcsat-server"
```

### Issue: Build timeout

**Cause**: Rust compilation takes >10 minutes

**Fix**: Enable build cache in Railway settings
```bash
# Or use cargo-chef for faster builds
```

### Issue: Database connection failed

**Fix**: Link PostgreSQL service in Railway dashboard
```bash
# Railway auto-injects DATABASE_URL
# Just add reference in railway.json:
{
  "services": {
    "backend": {
      "dependsOn": ["postgres", "redis"]
    }
  }
}
```

---

## 📊 Service Architecture

```
┌─────────────────────────────────────────────────┐
│               Railway Platform                  │
├─────────────────────────────────────────────────┤
│                                                 │
│  ┌──────────────┐  ┌──────────────┐  ┌────────┐│
│  │ Django App   │  │ Core (Django)│  │Backend ││
│  │ (Frontend)   │  │              │  │(Rust)  ││
│  │              │  │              │  │        ││
│  │ Port 8000    │  │ Port 8001    │  │Port 3000│
│  └──────┬───────┘  └──────┬───────┘  └────┬───┘│
│         │                 │               │    │
│         └─────────────────┼───────────────┘    │
│                           │                    │
│         ┌─────────────────┴─────────────────┐  │
│         │     PostgreSQL (Port 5432)        │  │
│         └───────────────────────────────────┘  │
│                                                 │
│         ┌───────────────────────────────────┐  │
│         │     Redis (Port 6379)             │  │
│         └───────────────────────────────────┘  │
└─────────────────────────────────────────────────┘
```

---

## 🔍 Health Check

Run diagnostics:
```powershell
.\check-railway-services.ps1
```

Expected output:
```
✅ Backend: 200 OK
✅ Core: 200 OK
✅ Django App: 200 OK
```

---

## 🎯 Next Steps

1. **Fix current 404s**:
   ```bash
   # Check Railway logs
   railway logs --service backend
   railway logs --service core
   railway logs --service djangoapp
   ```

2. **Verify builds**:
   - Backend should compile 5 Rust crates
   - Dockerfile should install Chrome
   - Binary should be at `/app/arcsat-server`

3. **Configure environment**:
   - Add DATABASE_URL (auto from PostgreSQL)
   - Add REDIS_URL (auto from Redis)
   - Set PORT=3000 for backend
   - Set RUST_LOG=info

4. **Test locally first**:
   ```powershell
   .\quickstart.ps1
   .\test-market-intelligence.ps1
   ```

5. **Deploy**:
   ```bash
   railway up
   ```

---

## 📝 Service URLs

After fixing deployment, services will be available at:

- **Backend API**: https://backend-production-5b7d.up.railway.app
  - `/api/v1/health`
  - `/api/v1/mi/jobs`
  - `/api/v1/crm/products/:id/insights`

- **Core API**: https://core-production-028a.up.railway.app
  - `/api/auth/`
  - `/api/erp/`

- **Frontend**: https://djangoapp-production-62bd.up.railway.app
  - `/dashboard`
  - `/login`

---

## 💰 Railway Costs

- **Starter Plan**: $5/month
- **Resources per service**:
  - 512 MB RAM
  - 1 GB storage
  - Shared CPU

- **Database add-ons**:
  - PostgreSQL: $5/month (1GB)
  - Redis: $5/month (256MB)

**Total estimated**: ~$20/month for 3 services + 2 databases
