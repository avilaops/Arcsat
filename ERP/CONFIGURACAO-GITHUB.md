# 🚀 Configuração Final do GitHub

## ✅ O que já foi feito:
- ✅ Código commitado e pushed para GitHub
- ✅ GitHub Actions configurado (.github/workflows/deploy.yml)
- ✅ Dockerfile e docker-compose prontos
- ✅ Frontend WASM compilável
- ✅ Backend Rust com MongoDB Atlas driver

---

## 🔧 O que VOCÊ precisa fazer agora:

### 1️⃣ Adicionar Secret do MongoDB Atlas

No repositório GitHub:
1. Vá em **Settings** → **Secrets and variables** → **Actions**
2. Clique em **New repository secret**
3. Adicione:
   - **Name:** `MONGO_ATLAS_URI`
   - **Secret:** Cole sua connection string do MongoDB Atlas
   - Formato: `mongodb+srv://usuario:senha@cluster.mongodb.net/erp?retryWrites=true&w=majority`

### 2️⃣ Configurar GitHub Pages

No repositório GitHub:
1. Vá em **Settings** → **Pages**
2. Em **Source**, selecione:
   - **Branch:** `gh-pages`
   - **Folder:** `/ (root)`
3. Clique em **Save**

⚠️ **Nota:** O branch `gh-pages` será criado automaticamente na primeira execução bem-sucedida do workflow.

### 3️⃣ Habilitar GitHub Actions (se necessário)

1. Vá em **Actions** (tab no topo do repositório)
2. Se aparecer um botão "I understand my workflows, go ahead and enable them", clique nele
3. O workflow "Deploy Avila ERP" deve aparecer

### 4️⃣ Verificar primeira execução

1. Em **Actions**, você verá o workflow rodando
2. Clique nele para ver os detalhes
3. Aguarde:
   - ✅ `deploy-frontend` - Compila WASM e faz deploy no GitHub Pages
   - ✅ `build-backend` - Cria imagem Docker e envia para GHCR

---

## 📍 Onde seu ERP estará após deploy:

### Frontend (GitHub Pages):
```
https://avilaops.github.io/Controle-Roncatin/
```

### Backend (Container no GHCR):
```
ghcr.io/avilaops/controle-roncatin/backend:latest
```

Para rodar o backend localmente:
```bash
docker pull ghcr.io/avilaops/controle-roncatin/backend:latest
docker run -p 3000:3000 -e MONGO_ATLAS_URI="sua-connection-string" ghcr.io/avilaops/controle-roncatin/backend:latest
```

---

## 🐛 Se algo der errado:

### 1. Frontend não compila?
- Verifique logs em **Actions** → workflow → `deploy-frontend`
- Pode ser necessário ajustar dependências no `Cargo.toml`

### 2. Docker build falha?
- Verifique se `MONGO_ATLAS_URI` está configurado nos Secrets
- Veja logs em **Actions** → workflow → `build-backend`

### 3. GitHub Pages não aparece?
- Aguarde 2-5 minutos após primeira execução
- Verifique se o branch `gh-pages` foi criado
- Confirme configuração em **Settings** → **Pages**

---

## 📞 Próximos Passos (Após Deploy):

1. **Testar Frontend:** Acesse `https://avilaops.github.io/Controle-Roncatin/`
2. **Verificar API:** Frontend vai tentar conectar em `http://localhost:3000/api`
3. **Ajustar URL da API:** Você precisará rodar o backend ou ajustar URL no frontend para apontar para onde você hospedará o backend
4. **Hospedar Backend:** Opções:
   - Railway.app (grátis)
   - Render.com (grátis)
   - Fly.io (grátis)
   - Azure Container Instances
   - AWS ECS
   - Google Cloud Run

---

## 🎯 Deploy Completo Final:

Para ter tudo funcionando em produção:

1. **Frontend:** ✅ GitHub Pages (já configurado)
2. **Backend:** Escolher plataforma (Railway/Render/Fly.io)
3. **Database:** ✅ MongoDB Atlas (já tem)
4. **Ajustar API URL:** No frontend, mudar de `localhost:3000` para URL do backend hospedado

---

## 📝 Checklist:

- [ ] Secret `MONGO_ATLAS_URI` adicionado
- [ ] GitHub Pages configurado (branch `gh-pages`)
- [ ] GitHub Actions habilitado
- [ ] Primeiro workflow executado com sucesso
- [ ] Frontend acessível em `https://avilaops.github.io/Controle-Roncatin/`
- [ ] Container backend disponível em GHCR
- [ ] Backend hospedado em alguma plataforma cloud
- [ ] Frontend ajustado para apontar para backend em produção

---

🎉 **Seu ERP estará no ar assim que completar essas etapas!**
