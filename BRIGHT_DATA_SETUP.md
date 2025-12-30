# Bright Data Proxy Configuration Guide

## 📝 O que é Bright Data?

Bright Data (antigo Luminati) é o maior provedor de proxies residenciais do mundo, com mais de 72 milhões de IPs reais. Essencial para scraping em escala.

## 🔑 Configuração

### 1. Criar Conta

1. Acesse: https://brightdata.com/
2. Crie uma conta (trial de 7 dias)
3. Navegue para Dashboard → Proxy Products

### 2. Criar Zona de Proxy

1. Clique em "Add Zone"
2. Escolha **"Residential Proxies"**
3. Configure:
   - **Zone Name**: `arcsat-scraping`
   - **Country**: Brazil (ou Multi-country)
   - **City**: All cities
   - **Session Type**: Rotating (recomendado)
   - **Session Duration**: 10 minutos

### 3. Obter Credenciais

Após criar a zona, você verá:
```
Host: brd.superproxy.io
Port: 22225
Username: brd-customer-xxxxxxxx-zone-arcsat-scraping
Password: seu-password-aqui
```

### 4. Configurar no Arcsat

Edite o `.env`:

```env
# Market Intelligence
MI_ENABLED=true
MI_PROXY_ENABLED=true
MI_PROXY_URL=http://brd-customer-xxxxxxxx-zone-arcsat-scraping:PASSWORD@brd.superproxy.io:22225
MI_MAX_CONCURRENT_JOBS=10

# Com proxy, podemos aumentar concorrência
SCRAPER_MAX_CONCURRENT=10
```

### 5. Testar Conexão

```powershell
# Testar proxy direto
curl -x http://brd-customer-xxx:pass@brd.superproxy.io:22225 https://lumtest.com/myip.json

# Deve retornar um IP residencial brasileiro
```

### 6. Rodar Worker com Proxy

```powershell
cd arcsat-backend
cargo run --bin arcsat-worker
```

Logs devem mostrar:
```
INFO arcsat_worker: ✅ Worker initialized (max concurrent: 10)
INFO arcsat_worker: 🎭 Proxy enabled
```

## 💰 Custos

### Planos Bright Data

| Plano | Tráfego | Custo | Requisições* |
|-------|---------|-------|-------------|
| Trial | 1 GB | $0 (7 dias) | ~5,000 |
| Starter | 40 GB | $500/mês | ~200,000 |
| Professional | 100 GB | $1,000/mês | ~500,000 |
| Enterprise | Custom | Negociar | Ilimitado |

*Estimativa: ~5KB por requisição (página HTML simples)

### Cálculo de Uso

Para **500 produtos/dia**:
- Amazon: ~50KB/produto = 25MB/dia = 750MB/mês
- Mercado Livre: ~30KB/produto = 15MB/dia = 450MB/mês
- **Total**: ~1.2GB/mês = **Trial suficiente para começar!**

Para **5000 produtos/dia**:
- ~12GB/mês = **Plano Starter ($500/mês)**

## 🎯 Configurações Avançadas

### Session Management

Para manter o mesmo IP durante uma sessão:

```env
MI_PROXY_URL=http://brd-customer-xxx-zone-arcsat-scraping-session-session1:pass@brd.superproxy.io:22225
```

Adicione `-session-sessionXXX` ao username para sticky sessions.

### Rotate on Failure

```env
# Rotate imediatamente se receber erro 429 (rate limit)
MI_PROXY_URL=http://brd-customer-xxx-zone-arcsat-scraping-country-br-session-random:pass@brd.superproxy.io:22225
```

### Targeting Específico

```env
# Apenas São Paulo
MI_PROXY_URL=http://brd-customer-xxx-zone-arcsat-scraping-country-br-city-saopaulo:pass@brd.superproxy.io:22225

# Apenas mobile
MI_PROXY_URL=http://brd-customer-xxx-zone-arcsat-scraping-mobile-true:pass@brd.superproxy.io:22225
```

## 🔍 Monitoramento

### Dashboard Bright Data

1. Acesse Dashboard → Statistics
2. Métricas importantes:
   - **Success Rate**: Deve ser >95%
   - **Bandwidth Usage**: Monitore consumo
   - **Requests**: Total de requisições
   - **Countries**: Distribuição geográfica

### Logs do Worker

```bash
# Ver logs em tempo real
tail -f worker.log

# Filtrar erros
grep ERROR worker.log

# Ver taxa de sucesso
grep "completed" worker.log | wc -l
```

## ⚠️ Troubleshooting

### Erro: "Authentication failed"
```
Verifique username/password no .env
Username deve incluir: brd-customer-XXXXX-zone-NOME
```

### Erro: "Connection timeout"
```
1. Verifique firewall (liberar porta 22225)
2. Teste conexão: telnet brd.superproxy.io 22225
3. Verifique saldo/créditos na conta
```

### Erro: "IP banned"
```
Mesmo com proxy, pode acontecer se:
1. Muitas requests simultâneas (reduza MI_MAX_CONCURRENT_JOBS)
2. Session duration muito longa (use rotating)
3. User-Agent suspeito (randomize no código)
```

### Success Rate Baixa (<90%)
```
Soluções:
1. Aumentar timeout: SCRAPER_TIMEOUT=60000
2. Adicionar delay: SCRAPER_DELAY_MIN=3
3. Usar sticky sessions para páginas sequenciais
4. Trocar de zona/IP pool
```

## 🚀 Alternativas Mais Baratas

### ScraperAPI (Mais Simples)
- URL: https://scraperapi.com/
- Custo: $49-99/mês
- Pros: API simples, mantém proxies
- Contras: Menos controle, limite de requests

```env
# ScraperAPI
MI_PROXY_URL=http://scraperapi:YOUR_API_KEY@proxy-server.scraperapi.com:8001
```

### Oxylabs (Alternativa)
- URL: https://oxylabs.io/
- Custo: $300/mês (starter)
- Similar ao Bright Data

```env
# Oxylabs
MI_PROXY_URL=http://customer-username:pass@pr.oxylabs.io:7777
```

### Smartproxy (Budget)
- URL: https://smartproxy.com/
- Custo: $75/mês (8GB)
- Boa para começar

```env
# Smartproxy
MI_PROXY_URL=http://user:pass@gate.smartproxy.com:7000
```

## 📊 Recomendação por Escala

| Escala | Volume/dia | Solução | Custo |
|--------|-----------|---------|-------|
| **MVP** | 0-500 produtos | Sem proxy (local) | $0 |
| **Teste** | 500-2000 produtos | Bright Data Trial | $0 (7 dias) |
| **Pequeno** | 2000-5000 produtos | ScraperAPI | $49-99/mês |
| **Médio** | 5000-20000 produtos | Bright Data Starter | $500/mês |
| **Grande** | 20000+ produtos | Bright Data Pro | $1000+/mês |

## 📝 Checklist de Deploy

- [ ] Conta Bright Data criada
- [ ] Zona de proxy configurada
- [ ] Credenciais testadas (curl)
- [ ] .env atualizado com MI_PROXY_URL
- [ ] Worker rodando com proxy enabled
- [ ] Monitoramento ativo (logs + dashboard)
- [ ] Alertas configurados (se usage > 80%)
- [ ] Backup plan (proxy alternativo)

---

**Pronto para escalar!** 🚀
