# 🚂 Deploy no Railway - Guia Rápido

## ✅ Pré-requisitos
- Conta no Railway (railway.app)
- Repositório GitHub conectado
- Variáveis de ambiente configuradas

## 📦 Configuração no Railway

### 1. Criar Novo Projeto
```bash
# Via Railway CLI (opcional)
railway login
railway init
railway link
```

### 2. Conectar GitHub
1. Acesse [railway.app/new](https://railway.app/new)
2. Selecione **"Deploy from GitHub repo"**
3. Escolha o repositório `avilaops/ERP`
4. Railway detectará automaticamente o `railway.json`

### 3. Configurar Variáveis de Ambiente

No painel do Railway, adicione as seguintes variáveis:

#### ⚙️ Essenciais
```bash
DATABASE_URL=sqlite:///app/database/erp.db
PORT=3000
HOST=0.0.0.0
RUST_LOG=info
JWT_SECRET=<gere_uma_chave_segura_aqui>
ENVIRONMENT=production
```

#### 🔐 APIs Externas (Copiar do .env local)
```bash
OPENAI_API_KEY=sk-proj-...
DEEPSEEK_API_KEY=sk-...
STRIPE_API=rk_test_...
PAYPAL_ID=...
PAYPAL_TOKEN_API=...
CLOUDFLARE_API_KEY=...
GCLOUD_API_TOKEN=...
MONGO_ATLAS_URI=mongodb+srv://...
```

### 4. Adicionar Serviços (Plugins Railway)

#### PostgreSQL (Recomendado para produção)
```bash
railway add postgresql
```
Isso criará automaticamente a variável `DATABASE_URL`.

**Atualizar backend para usar PostgreSQL:**
```toml
# backend/Cargo.toml
[dependencies]
sqlx = { version = "0.8", features = ["postgres", "runtime-tokio-rustls", "uuid", "chrono"] }
```

#### Redis (Cache - Opcional)
```bash
railway add redis
```
Isso criará automaticamente a variável `REDIS_URL`.

### 5. Build e Deploy

Railway iniciará o build automaticamente:

1. **Detecção**: Usa `Dockerfile.railway`
2. **Build**: Compila Rust em modo release
3. **Deploy**: Inicia container na porta 3000
4. **URL**: Railway fornecerá uma URL pública (ex: `erp-production.up.railway.app`)

### 6. Verificar Deploy

```bash
# Via CLI
railway logs

# Via Web
https://railway.app/project/<project-id>/service/<service-id>
```

Endpoints disponíveis:
- ✅ Health: `https://seu-app.up.railway.app/health`
- 📊 API: `https://seu-app.up.railway.app/api/v1`

## 🔧 Configurações Avançadas

### Custom Domain
```bash
railway domain
```

### Variáveis de Build
```json
// railway.json
{
  "build": {
    "builder": "DOCKERFILE",
    "dockerfilePath": "Dockerfile.railway"
  }
}
```

### Health Checks
Railway monitora automaticamente a porta 3000.

### Logs e Monitoramento
```bash
railway logs --follow
```

## 🐛 Troubleshooting

### Build Falha
- Verifique se `Cargo.toml` está correto
- Confirme que todas as dependências compilam localmente

### App Não Inicia
- Verifique logs: `railway logs`
- Confirme que `DATABASE_URL` está configurado
- Teste localmente com as mesmas variáveis

### Erro 503
- Verifique se a porta 3000 está exposta
- Confirme que o processo está rodando

## 📚 Recursos

- [Railway Docs](https://docs.railway.app)
- [Rust on Railway](https://docs.railway.app/guides/rust)
- [Environment Variables](https://docs.railway.app/develop/variables)

## 🚀 Próximos Passos

1. ✅ Deploy backend funcionando
2. 🔜 Adicionar PostgreSQL para produção
3. 🔜 Configurar Redis para cache
4. 🔜 Deploy frontend (Vercel/Netlify)
5. 🔜 Configurar domínio customizado
6. 🔜 Adicionar CI/CD com GitHub Actions

---

**Status Atual:** Backend pronto para deploy no Railway! 🎉
