# 🔌 ERP/CRM Faria Lima - API Reference

## 📘 Visão Geral da API

**Base URL Production**: `https://api.erp-faria-lima.com/v1`  
**Base URL Staging**: `https://staging-api.erp-faria-lima.com/v1`

**Autenticação**: Bearer Token (JWT)  
**Rate Limit**: 1000 requisições/minuto por tenant  
**Formato**: JSON (Content-Type: application/json)

---

## 🔐 Autenticação

### Login
```http
POST /auth/login
Content-Type: application/json

{
  "email": "usuario@empresa.com",
  "password": "senha_segura",
  "tenant_domain": "empresa.erp.com"
}

Response 200:
{
  "access_token": "eyJhbGciOiJIUzI1NiIs...",
  "refresh_token": "dGhpc2lzYXJlZnJlc2h...",
  "expires_in": 3600,
  "user": {
    "id": "uuid",
    "name": "João Silva",
    "email": "usuario@empresa.com",
    "roles": ["sales_manager", "admin"],
    "tenant_id": "tenant-uuid"
  }
}
```

### SSO (Single Sign-On)
```http
GET /auth/sso/google
GET /auth/sso/microsoft
GET /auth/sso/linkedin

Response: Redirect to provider
```

### Refresh Token
```http
POST /auth/refresh
Content-Type: application/json

{
  "refresh_token": "dGhpc2lzYXJlZnJlc2h..."
}
```

---

## 💼 CRM APIs

### Leads

#### Criar Lead
```http
POST /crm/leads
Authorization: Bearer {token}
Content-Type: application/json

{
  "name": "Maria Oliveira",
  "company": "Tech Corp LTDA",
  "email": "maria@techcorp.com",
  "phone": "+5511999887766",
  "source": "linkedin",
  "stage": "qualification",
  "value": 150000.00,
  "probability": 20,
  "expected_close_date": "2024-12-31",
  "custom_fields": {
    "industry": "technology",
    "employees": 250,
    "current_erp": "SAP"
  }
}

Response 201:
{
  "id": "lead-uuid",
  "name": "Maria Oliveira",
  "stage": "qualification",
  "score": 65,
  "owner": {
    "id": "user-uuid",
    "name": "João Vendedor"
  },
  "created_at": "2024-01-15T10:30:00Z"
}
```

#### Listar Leads com Filtros
```http
GET /crm/leads?stage=qualification&score_min=50&page=1&limit=50
Authorization: Bearer {token}

Response 200:
{
  "data": [
    {
      "id": "lead-uuid",
      "name": "Maria Oliveira",
      "company": "Tech Corp LTDA",
      "stage": "qualification",
      "score": 65,
      "value": 150000.00,
      "last_contact": "2024-01-14T15:20:00Z"
    }
  ],
  "pagination": {
    "current_page": 1,
    "total_pages": 10,
    "total_items": 487,
    "per_page": 50
  }
}
```

#### Mover Lead no Pipeline
```http
PATCH /crm/leads/{id}/stage
Authorization: Bearer {token}
Content-Type: application/json

{
  "stage": "proposal",
  "reason": "Cliente aprovou orçamento inicial",
  "scheduled_call": "2024-01-20T14:00:00Z"
}

Response 200:
{
  "id": "lead-uuid",
  "stage": "proposal",
  "probability": 60,
  "history": [
    {
      "stage": "qualification",
      "duration_days": 7,
      "changed_at": "2024-01-08T10:00:00Z"
    }
  ]
}
```

### Oportunidades

#### Forecast de Vendas
```http
GET /crm/opportunities/forecast?period=Q1-2024&team_id=sales-team-uuid
Authorization: Bearer {token}

Response 200:
{
  "period": "Q1-2024",
  "forecast": {
    "best_case": 2500000.00,
    "most_likely": 1850000.00,
    "worst_case": 1200000.00,
    "weighted": 1765000.00
  },
  "breakdown_by_stage": {
    "qualification": { "count": 45, "value": 1200000.00, "probability": 20 },
    "proposal": { "count": 23, "value": 890000.00, "probability": 60 },
    "negotiation": { "count": 12, "value": 675000.00, "probability": 80 }
  }
}
```

### Contas (Accounts)

#### Health Score do Cliente
```http
GET /crm/accounts/{id}/health-score
Authorization: Bearer {token}

Response 200:
{
  "account_id": "account-uuid",
  "health_score": 78,
  "status": "healthy",
  "factors": {
    "engagement": { "score": 85, "weight": 30, "trend": "up" },
    "financial": { "score": 70, "weight": 25, "trend": "stable" },
    "support_tickets": { "score": 80, "weight": 20, "trend": "up" },
    "product_usage": { "score": 75, "weight": 25, "trend": "down" }
  },
  "risk_indicators": [
    {
      "type": "low_usage",
      "severity": "medium",
      "message": "Uso do produto caiu 15% no último mês"
    }
  ],
  "recommendations": [
    "Agendar quarterly business review",
    "Oferecer treinamento avançado"
  ]
}
```

---

## 💰 Financeiro APIs

### Fluxo de Caixa

#### Projeção Preditiva
```http
POST /finance/cashflow/projection
Authorization: Bearer {token}
Content-Type: application/json

{
  "start_date": "2024-02-01",
  "months": 12,
  "scenario": "realistic",
  "include_ai_prediction": true
}

Response 200:
{
  "projection": [
    {
      "month": "2024-02",
      "opening_balance": 500000.00,
      "inflows": {
        "sales": 350000.00,
        "receivables": 120000.00,
        "other": 10000.00
      },
      "outflows": {
        "payroll": 180000.00,
        "suppliers": 95000.00,
        "taxes": 45000.00,
        "operational": 60000.00
      },
      "closing_balance": 600000.00,
      "ai_confidence": 0.87
    }
  ],
  "scenarios": {
    "optimistic": { "closing_balance_dec": 1250000.00 },
    "realistic": { "closing_balance_dec": 890000.00 },
    "pessimistic": { "closing_balance_dec": 450000.00 }
  },
  "alerts": [
    {
      "month": "2024-07",
      "type": "low_balance",
      "message": "Saldo projetado abaixo do mínimo (R$ 200k)"
    }
  ]
}
```

### DRE Gerencial

#### DRE em Tempo Real
```http
GET /finance/dre/realtime?start=2024-01-01&end=2024-12-31&group_by=month
Authorization: Bearer {token}

Response 200:
{
  "period": "2024",
  "summary": {
    "revenue": {
      "gross": 12500000.00,
      "net": 11875000.00,
      "growth_yoy": 23.5
    },
    "costs": {
      "cogs": 4500000.00,
      "gross_margin": 62.0
    },
    "expenses": {
      "sales": 2100000.00,
      "administrative": 1850000.00,
      "operational": 1200000.00
    },
    "ebitda": 2225000.00,
    "ebitda_margin": 18.7,
    "net_income": 1456000.00,
    "net_margin": 12.3
  },
  "monthly": [
    {
      "month": "2024-01",
      "revenue": 980000.00,
      "ebitda": 178000.00,
      "margin": 18.2
    }
  ]
}
```

### Pagamentos

#### Criar Pagamento PIX
```http
POST /finance/payments/pix
Authorization: Bearer {token}
Content-Type: application/json

{
  "type": "pix_key",
  "pix_key": "empresa@exemplo.com",
  "amount": 15000.50,
  "description": "Pagamento fornecedor XYZ - NF 12345",
  "scheduled_date": "2024-01-20",
  "account_payable_id": "ap-uuid",
  "approval_workflow": true
}

Response 201:
{
  "id": "payment-uuid",
  "status": "pending_approval",
  "pix_qrcode": "00020126580014br.gov.bcb.pix...",
  "pix_copy_paste": "00020126580014br.gov.bcb.pix...",
  "expires_at": "2024-01-20T23:59:59Z",
  "approval_chain": [
    { "approver": "manager-uuid", "status": "pending" },
    { "approver": "cfo-uuid", "status": "pending" }
  ]
}
```

### Notas Fiscais

#### Emitir NFe
```http
POST /finance/invoices/nfe
Authorization: Bearer {token}
Content-Type: application/json

{
  "customer": {
    "cnpj": "12345678000190",
    "name": "Cliente Exemplo LTDA",
    "address": {
      "street": "Av. Faria Lima",
      "number": "3000",
      "city": "São Paulo",
      "state": "SP",
      "zipcode": "01452000"
    }
  },
  "items": [
    {
      "code": "PROD-001",
      "description": "Licença Software ERP - Anual",
      "quantity": 1,
      "unit_price": 50000.00,
      "tax_rate": 0.05,
      "ncm": "85234910"
    }
  ],
  "payment": {
    "method": "bank_slip",
    "due_date": "2024-02-15"
  }
}

Response 201:
{
  "id": "nfe-uuid",
  "number": "000123456",
  "series": "1",
  "access_key": "35240112345678000190550010001234561123456780",
  "status": "authorized",
  "xml_url": "https://storage.../nfe-123456.xml",
  "pdf_url": "https://storage.../nfe-123456.pdf",
  "issued_at": "2024-01-15T14:32:00Z"
}
```

---

## 👥 Recursos Humanos APIs

### Funcionários

#### Listar Funcionários
```http
GET /hr/employees?department=sales&status=active&page=1&limit=50
Authorization: Bearer {token}

Response 200:
{
  "data": [
    {
      "id": "employee-uuid",
      "name": "Carlos Santos",
      "email": "carlos@empresa.com",
      "department": "sales",
      "position": "Sales Manager",
      "admission_date": "2022-03-15",
      "salary": 12000.00,
      "performance_score": 4.5
    }
  ],
  "pagination": { "current_page": 1, "total_pages": 3, "total_items": 124 }
}
```

### Folha de Pagamento

#### Calcular Folha
```http
POST /hr/payroll/calculate
Authorization: Bearer {token}
Content-Type: application/json

{
  "reference_month": "2024-01",
  "employees": ["employee-uuid-1", "employee-uuid-2"],
  "include_benefits": true,
  "include_overtime": true
}

Response 200:
{
  "reference_month": "2024-01",
  "total_gross": 450000.00,
  "total_deductions": 98500.00,
  "total_net": 351500.00,
  "employer_charges": {
    "inss": 67500.00,
    "fgts": 36000.00,
    "total": 103500.00
  },
  "employees": [
    {
      "id": "employee-uuid-1",
      "name": "Carlos Santos",
      "gross_salary": 12000.00,
      "overtime": 800.00,
      "benefits": 1200.00,
      "deductions": {
        "inss": 1320.00,
        "irrf": 856.00,
        "health_plan": 450.00
      },
      "net_salary": 11374.00
    }
  ]
}
```

### Avaliação de Desempenho

#### Criar Avaliação 360°
```http
POST /hr/performance-review
Authorization: Bearer {token}
Content-Type: application/json

{
  "employee_id": "employee-uuid",
  "period": "2024-H1",
  "type": "360",
  "evaluators": [
    { "user_id": "manager-uuid", "type": "manager" },
    { "user_id": "peer1-uuid", "type": "peer" },
    { "user_id": "peer2-uuid", "type": "peer" },
    { "user_id": "subordinate1-uuid", "type": "subordinate" }
  ],
  "competencies": [
    { "name": "Liderança", "weight": 25 },
    { "name": "Comunicação", "weight": 20 },
    { "name": "Resultados", "weight": 30 },
    { "name": "Trabalho em equipe", "weight": 25 }
  ]
}

Response 201:
{
  "id": "review-uuid",
  "status": "in_progress",
  "deadline": "2024-07-15",
  "notifications_sent": true
}
```

---

## 📦 Supply Chain APIs

### Estoque

#### Previsão de Demanda
```http
GET /inventory/forecast?product_id=prod-uuid&months=6&algorithm=ml
Authorization: Bearer {token}

Response 200:
{
  "product_id": "prod-uuid",
  "product_name": "Widget Premium",
  "current_stock": 1250,
  "reorder_point": 500,
  "forecast": [
    {
      "month": "2024-02",
      "predicted_demand": 380,
      "confidence_interval": { "lower": 320, "upper": 440 },
      "recommended_purchase": 400,
      "confidence": 0.89
    }
  ],
  "insights": [
    "Padrão sazonal detectado: aumento de 30% em Q4",
    "Tendência de crescimento: +15% ano/ano"
  ]
}
```

### Compras

#### Criar RFQ (Request for Quotation)
```http
POST /purchasing/rfq
Authorization: Bearer {token}
Content-Type: application/json

{
  "title": "Cotação Notebooks - 50 unidades",
  "description": "Notebooks i7, 16GB RAM, 512GB SSD",
  "items": [
    {
      "product_name": "Notebook Dell Latitude 5430",
      "quantity": 50,
      "specifications": "i7-12650H, 16GB, 512GB SSD",
      "delivery_location": "São Paulo - SP"
    }
  ],
  "suppliers": ["supplier-uuid-1", "supplier-uuid-2", "supplier-uuid-3"],
  "deadline": "2024-01-25T18:00:00Z",
  "delivery_date_required": "2024-02-15"
}

Response 201:
{
  "id": "rfq-uuid",
  "status": "sent",
  "suppliers_notified": 3,
  "quotes_received": 0,
  "deadline": "2024-01-25T18:00:00Z"
}
```

---

## 📊 Analytics APIs

### KPIs

#### Dashboard Financeiro
```http
GET /analytics/kpis/financial?period=2024-01&compare_previous=true
Authorization: Bearer {token}

Response 200:
{
  "period": "2024-01",
  "mrr": {
    "value": 850000.00,
    "change": 5.2,
    "trend": "up"
  },
  "arr": {
    "value": 10200000.00,
    "change": 23.5,
    "trend": "up"
  },
  "cac": {
    "value": 12500.00,
    "change": -8.3,
    "trend": "down"
  },
  "ltv": {
    "value": 95000.00,
    "change": 12.1,
    "trend": "up"
  },
  "ltv_cac_ratio": 7.6,
  "churn_rate": {
    "value": 2.3,
    "change": -0.5,
    "trend": "down"
  },
  "burn_rate": {
    "value": 180000.00,
    "runway_months": 18.5
  }
}
```

### Relatórios Customizados

#### Criar Dashboard Personalizado
```http
POST /analytics/dashboards/custom
Authorization: Bearer {token}
Content-Type: application/json

{
  "name": "Dashboard Vendas - Diretoria",
  "description": "KPIs executivos de vendas",
  "widgets": [
    {
      "type": "metric",
      "title": "MRR Atual",
      "query": "SELECT SUM(mrr) FROM subscriptions WHERE status='active'",
      "visualization": "number",
      "position": { "x": 0, "y": 0, "width": 3, "height": 2 }
    },
    {
      "type": "chart",
      "title": "Pipeline por Estágio",
      "query": "SELECT stage, COUNT(*), SUM(value) FROM opportunities GROUP BY stage",
      "visualization": "funnel",
      "position": { "x": 3, "y": 0, "width": 6, "height": 4 }
    }
  ],
  "refresh_interval": 300,
  "share_with": ["sales-team-uuid"]
}

Response 201:
{
  "id": "dashboard-uuid",
  "url": "/dashboards/custom/dashboard-uuid",
  "share_url": "https://app.erp.com/public/dash/abc123xyz"
}
```

---

## 🔔 Webhooks

### Configurar Webhook
```http
POST /webhooks
Authorization: Bearer {token}
Content-Type: application/json

{
  "url": "https://sua-empresa.com/webhooks/erp",
  "events": [
    "lead.created",
    "opportunity.stage_changed",
    "invoice.paid",
    "employee.hired"
  ],
  "secret": "webhook_secret_key_aqui"
}

Response 201:
{
  "id": "webhook-uuid",
  "status": "active",
  "events": ["lead.created", "opportunity.stage_changed", "invoice.paid", "employee.hired"]
}
```

### Payload Exemplo
```json
{
  "event": "opportunity.stage_changed",
  "timestamp": "2024-01-15T10:30:00Z",
  "data": {
    "opportunity_id": "opp-uuid",
    "previous_stage": "qualification",
    "new_stage": "proposal",
    "owner": {
      "id": "user-uuid",
      "name": "João Vendedor"
    },
    "value": 150000.00,
    "probability": 60
  },
  "signature": "sha256=abc123..."
}
```

---

## ⚠️ Códigos de Erro

```json
{
  "error": {
    "code": "VALIDATION_ERROR",
    "message": "Os dados fornecidos são inválidos",
    "details": [
      {
        "field": "email",
        "message": "Email inválido"
      }
    ]
  }
}
```

### Códigos Comuns:
- `400` - Bad Request (validação)
- `401` - Unauthorized (token inválido)
- `403` - Forbidden (sem permissão)
- `404` - Not Found
- `429` - Rate Limit Exceeded
- `500` - Internal Server Error
- `503` - Service Unavailable

---

## 🚀 SDKs Disponíveis

### JavaScript/TypeScript
```bash
npm install @erp-faria-lima/sdk
```

```typescript
import { ERPClient } from '@erp-faria-lima/sdk';

const client = new ERPClient({
  apiKey: 'seu-api-key',
  baseURL: 'https://api.erp-faria-lima.com/v1'
});

// Criar lead
const lead = await client.crm.leads.create({
  name: 'Maria Silva',
  email: 'maria@empresa.com',
  stage: 'qualification'
});
```

### Python
```bash
pip install erp-faria-lima
```

```python
from erp_faria_lima import ERPClient

client = ERPClient(api_key='seu-api-key')

# Listar oportunidades
opportunities = client.crm.opportunities.list(
    stage='proposal',
    min_value=50000
)
```

### Rust
```toml
[dependencies]
erp-faria-lima = "0.1.0"
```

```rust
use erp_faria_lima::{ERPClient, CRMLead};

let client = ERPClient::new("seu-api-key");

let lead = CRMLead {
    name: "João Silva".to_string(),
    email: "joao@empresa.com".to_string(),
    // ...
};

let result = client.crm().leads().create(lead).await?;
```

---

## 📚 Recursos Adicionais

- **Documentação Interativa**: https://docs.erp-faria-lima.com
- **Postman Collection**: [Download](https://api.erp-faria-lima.com/postman)
- **Status Page**: https://status.erp-faria-lima.com
- **Changelog**: https://docs.erp-faria-lima.com/changelog
- **Suporte**: suporte@erp-faria-lima.com

---

**Versão da API**: v1.0.0  
**Última Atualização**: Janeiro 2024
