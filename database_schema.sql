-- ============================================================================
-- SCHEMA SQL - ERP/CRM Faria Lima
-- PostgreSQL 15+
-- ============================================================================

-- Habilitar extensões
CREATE EXTENSION IF NOT EXISTS "uuid-ossp";
CREATE EXTENSION IF NOT EXISTS "pg_trgm"; -- Para busca fuzzy
CREATE EXTENSION IF NOT EXISTS "btree_gin"; -- Para índices compostos

-- ============================================================================
-- MULTI-TENANCY
-- ============================================================================

CREATE TABLE tenants (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    name VARCHAR(255) NOT NULL,
    domain VARCHAR(100) UNIQUE NOT NULL,
    plan VARCHAR(50) NOT NULL DEFAULT 'startup', -- startup, business, enterprise
    status VARCHAR(20) NOT NULL DEFAULT 'active', -- active, suspended, canceled
    settings JSONB DEFAULT '{}',
    max_users INTEGER DEFAULT 50,
    storage_limit_gb INTEGER DEFAULT 100,
    created_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP
);

CREATE INDEX idx_tenants_domain ON tenants(domain);
CREATE INDEX idx_tenants_status ON tenants(status) WHERE status = 'active';

-- ============================================================================
-- USUÁRIOS E AUTENTICAÇÃO
-- ============================================================================

CREATE TABLE users (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    tenant_id UUID NOT NULL REFERENCES tenants(id) ON DELETE CASCADE,
    email VARCHAR(255) NOT NULL,
    password_hash VARCHAR(255), -- NULL para SSO
    name VARCHAR(255) NOT NULL,
    avatar_url TEXT,
    phone VARCHAR(20),
    language VARCHAR(5) DEFAULT 'pt-BR',
    timezone VARCHAR(50) DEFAULT 'America/Sao_Paulo',
    status VARCHAR(20) DEFAULT 'active', -- active, inactive, suspended
    last_login_at TIMESTAMP WITH TIME ZONE,
    created_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP,
    UNIQUE(tenant_id, email)
);

CREATE INDEX idx_users_tenant ON users(tenant_id);
CREATE INDEX idx_users_email ON users(email);

-- Roles e Permissões (RBAC)
CREATE TABLE roles (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    tenant_id UUID NOT NULL REFERENCES tenants(id) ON DELETE CASCADE,
    name VARCHAR(100) NOT NULL,
    description TEXT,
    permissions JSONB NOT NULL DEFAULT '[]',
    is_system BOOLEAN DEFAULT false, -- admin, user, etc
    created_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP,
    UNIQUE(tenant_id, name)
);

CREATE TABLE user_roles (
    user_id UUID REFERENCES users(id) ON DELETE CASCADE,
    role_id UUID REFERENCES roles(id) ON DELETE CASCADE,
    assigned_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP,
    PRIMARY KEY (user_id, role_id)
);

-- ============================================================================
-- CRM - LEADS & OPORTUNIDADES
-- ============================================================================

CREATE TYPE lead_stage AS ENUM (
    'new',
    'contacted',
    'qualification',
    'proposal',
    'negotiation',
    'won',
    'lost'
);

CREATE TYPE lead_source AS ENUM (
    'website',
    'linkedin',
    'referral',
    'cold_call',
    'event',
    'partner',
    'other'
);

CREATE TABLE crm_leads (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    tenant_id UUID NOT NULL REFERENCES tenants(id) ON DELETE CASCADE,
    name VARCHAR(255) NOT NULL,
    company VARCHAR(255),
    email VARCHAR(255) NOT NULL,
    phone VARCHAR(20),
    source lead_source NOT NULL,
    stage lead_stage DEFAULT 'new',
    score INTEGER DEFAULT 50 CHECK (score >= 0 AND score <= 100),
    value DECIMAL(15, 2) DEFAULT 0,
    probability INTEGER DEFAULT 0 CHECK (probability >= 0 AND probability <= 100),
    owner_id UUID REFERENCES users(id) ON DELETE SET NULL,
    expected_close_date DATE,
    actual_close_date DATE,
    lost_reason TEXT,
    custom_fields JSONB DEFAULT '{}',
    created_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP
);

CREATE INDEX idx_leads_tenant ON crm_leads(tenant_id);
CREATE INDEX idx_leads_stage ON crm_leads(stage);
CREATE INDEX idx_leads_owner ON crm_leads(owner_id);
CREATE INDEX idx_leads_score ON crm_leads(score DESC);
CREATE INDEX idx_leads_email ON crm_leads USING gin(email gin_trgm_ops);

-- Histórico de mudanças de estágio
CREATE TABLE crm_lead_stage_history (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    lead_id UUID NOT NULL REFERENCES crm_leads(id) ON DELETE CASCADE,
    from_stage lead_stage,
    to_stage lead_stage NOT NULL,
    reason TEXT,
    changed_by UUID REFERENCES users(id),
    duration_seconds INTEGER,
    changed_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP
);

CREATE INDEX idx_lead_history_lead ON crm_lead_stage_history(lead_id);

-- Contas (Clientes)
CREATE TABLE crm_accounts (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    tenant_id UUID NOT NULL REFERENCES tenants(id) ON DELETE CASCADE,
    name VARCHAR(255) NOT NULL,
    cnpj VARCHAR(18) UNIQUE,
    industry VARCHAR(100),
    employees_count INTEGER,
    annual_revenue DECIMAL(15, 2),
    website TEXT,
    address JSONB, -- {street, number, city, state, zipcode}
    account_manager_id UUID REFERENCES users(id),
    health_score INTEGER CHECK (health_score >= 0 AND health_score <= 100),
    status VARCHAR(20) DEFAULT 'active', -- active, at_risk, churned
    custom_fields JSONB DEFAULT '{}',
    created_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP
);

CREATE INDEX idx_accounts_tenant ON crm_accounts(tenant_id);
CREATE INDEX idx_accounts_cnpj ON crm_accounts(cnpj);
CREATE INDEX idx_accounts_health ON crm_accounts(health_score);

-- Contatos
CREATE TABLE crm_contacts (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    tenant_id UUID NOT NULL REFERENCES tenants(id) ON DELETE CASCADE,
    account_id UUID REFERENCES crm_accounts(id) ON DELETE SET NULL,
    name VARCHAR(255) NOT NULL,
    email VARCHAR(255),
    phone VARCHAR(20),
    position VARCHAR(100),
    department VARCHAR(100),
    is_decision_maker BOOLEAN DEFAULT false,
    linkedin_url TEXT,
    custom_fields JSONB DEFAULT '{}',
    created_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP
);

CREATE INDEX idx_contacts_tenant ON crm_contacts(tenant_id);
CREATE INDEX idx_contacts_account ON crm_contacts(account_id);
CREATE INDEX idx_contacts_email ON crm_contacts(email);

-- Atividades (Tarefas, Reuniões, Ligações)
CREATE TYPE activity_type AS ENUM ('task', 'call', 'meeting', 'email', 'note');
CREATE TYPE activity_status AS ENUM ('scheduled', 'completed', 'canceled', 'overdue');

CREATE TABLE crm_activities (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    tenant_id UUID NOT NULL REFERENCES tenants(id) ON DELETE CASCADE,
    type activity_type NOT NULL,
    subject VARCHAR(255) NOT NULL,
    description TEXT,
    status activity_status DEFAULT 'scheduled',
    scheduled_at TIMESTAMP WITH TIME ZONE,
    completed_at TIMESTAMP WITH TIME ZONE,
    duration_minutes INTEGER,
    lead_id UUID REFERENCES crm_leads(id) ON DELETE CASCADE,
    account_id UUID REFERENCES crm_accounts(id) ON DELETE CASCADE,
    contact_id UUID REFERENCES crm_contacts(id) ON DELETE SET NULL,
    owner_id UUID REFERENCES users(id),
    attendees UUID[], -- Array de user IDs
    custom_fields JSONB DEFAULT '{}',
    created_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP
);

CREATE INDEX idx_activities_tenant ON crm_activities(tenant_id);
CREATE INDEX idx_activities_owner ON crm_activities(owner_id);
CREATE INDEX idx_activities_scheduled ON crm_activities(scheduled_at) WHERE status = 'scheduled';

-- ============================================================================
-- FINANCEIRO
-- ============================================================================

CREATE TYPE transaction_type AS ENUM ('revenue', 'expense', 'transfer');
CREATE TYPE payment_status AS ENUM ('pending', 'approved', 'paid', 'canceled', 'overdue');
CREATE TYPE payment_method AS ENUM ('pix', 'bank_transfer', 'bank_slip', 'credit_card', 'cash', 'other');

-- Contas Bancárias
CREATE TABLE finance_bank_accounts (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    tenant_id UUID NOT NULL REFERENCES tenants(id) ON DELETE CASCADE,
    name VARCHAR(255) NOT NULL,
    bank_code VARCHAR(10),
    bank_name VARCHAR(100),
    agency VARCHAR(20),
    account_number VARCHAR(20),
    account_type VARCHAR(20), -- checking, savings
    balance DECIMAL(15, 2) DEFAULT 0,
    is_active BOOLEAN DEFAULT true,
    created_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP
);

CREATE INDEX idx_bank_accounts_tenant ON finance_bank_accounts(tenant_id);

-- Transações Financeiras
CREATE TABLE finance_transactions (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    tenant_id UUID NOT NULL REFERENCES tenants(id) ON DELETE CASCADE,
    type transaction_type NOT NULL,
    category VARCHAR(100), -- sales, payroll, rent, etc
    cost_center VARCHAR(100),
    description TEXT NOT NULL,
    amount DECIMAL(15, 2) NOT NULL,
    date DATE NOT NULL,
    bank_account_id UUID REFERENCES finance_bank_accounts(id),
    reference_id UUID, -- Referência para AP/AR
    custom_fields JSONB DEFAULT '{}',
    created_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP
);

CREATE INDEX idx_transactions_tenant ON finance_transactions(tenant_id);
CREATE INDEX idx_transactions_date ON finance_transactions(date);
CREATE INDEX idx_transactions_type ON finance_transactions(type);
CREATE INDEX idx_transactions_category ON finance_transactions(category);

-- Contas a Pagar
CREATE TABLE finance_accounts_payable (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    tenant_id UUID NOT NULL REFERENCES tenants(id) ON DELETE CASCADE,
    supplier_id UUID, -- Referência a fornecedores
    invoice_number VARCHAR(100),
    description TEXT NOT NULL,
    amount DECIMAL(15, 2) NOT NULL,
    due_date DATE NOT NULL,
    payment_date DATE,
    payment_method payment_method,
    status payment_status DEFAULT 'pending',
    category VARCHAR(100),
    cost_center VARCHAR(100),
    notes TEXT,
    attachment_url TEXT,
    created_by UUID REFERENCES users(id),
    approved_by UUID REFERENCES users(id),
    created_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP
);

CREATE INDEX idx_ap_tenant ON finance_accounts_payable(tenant_id);
CREATE INDEX idx_ap_due_date ON finance_accounts_payable(due_date);
CREATE INDEX idx_ap_status ON finance_accounts_payable(status);

-- Contas a Receber
CREATE TABLE finance_accounts_receivable (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    tenant_id UUID NOT NULL REFERENCES tenants(id) ON DELETE CASCADE,
    customer_id UUID REFERENCES crm_accounts(id),
    invoice_number VARCHAR(100),
    description TEXT NOT NULL,
    amount DECIMAL(15, 2) NOT NULL,
    due_date DATE NOT NULL,
    payment_date DATE,
    payment_method payment_method,
    status payment_status DEFAULT 'pending',
    notes TEXT,
    nfe_key VARCHAR(44), -- Chave de acesso NFe
    nfe_url TEXT,
    created_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP
);

CREATE INDEX idx_ar_tenant ON finance_accounts_receivable(tenant_id);
CREATE INDEX idx_ar_due_date ON finance_accounts_receivable(due_date);
CREATE INDEX idx_ar_status ON finance_accounts_receivable(status);
CREATE INDEX idx_ar_customer ON finance_accounts_receivable(customer_id);

-- Orçamento (Budget)
CREATE TABLE finance_budgets (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    tenant_id UUID NOT NULL REFERENCES tenants(id) ON DELETE CASCADE,
    year INTEGER NOT NULL,
    month INTEGER CHECK (month >= 1 AND month <= 12),
    category VARCHAR(100) NOT NULL,
    cost_center VARCHAR(100),
    planned_amount DECIMAL(15, 2) NOT NULL,
    actual_amount DECIMAL(15, 2) DEFAULT 0,
    created_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP,
    UNIQUE(tenant_id, year, month, category, cost_center)
);

CREATE INDEX idx_budgets_tenant ON finance_budgets(tenant_id);
CREATE INDEX idx_budgets_period ON finance_budgets(year, month);

-- ============================================================================
-- RECURSOS HUMANOS
-- ============================================================================

CREATE TYPE employment_type AS ENUM ('clt', 'pj', 'intern', 'contractor');
CREATE TYPE employee_status AS ENUM ('active', 'on_leave', 'resigned', 'terminated');

CREATE TABLE hr_employees (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    tenant_id UUID NOT NULL REFERENCES tenants(id) ON DELETE CASCADE,
    user_id UUID REFERENCES users(id), -- Vincula com usuário do sistema
    full_name VARCHAR(255) NOT NULL,
    cpf VARCHAR(14) UNIQUE NOT NULL,
    rg VARCHAR(20),
    birth_date DATE,
    email VARCHAR(255),
    phone VARCHAR(20),
    address JSONB,
    employment_type employment_type NOT NULL,
    status employee_status DEFAULT 'active',
    department VARCHAR(100),
    position VARCHAR(100),
    manager_id UUID REFERENCES hr_employees(id),
    admission_date DATE NOT NULL,
    resignation_date DATE,
    base_salary DECIMAL(12, 2),
    benefits JSONB DEFAULT '{}', -- {health_plan: 450, meal_voucher: 800}
    bank_info JSONB, -- {bank, agency, account}
    performance_score DECIMAL(3, 2) CHECK (performance_score >= 0 AND performance_score <= 5),
    custom_fields JSONB DEFAULT '{}',
    created_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP
);

CREATE INDEX idx_employees_tenant ON hr_employees(tenant_id);
CREATE INDEX idx_employees_cpf ON hr_employees(cpf);
CREATE INDEX idx_employees_status ON hr_employees(status);
CREATE INDEX idx_employees_department ON hr_employees(department);

-- Folha de Pagamento
CREATE TABLE hr_payroll (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    tenant_id UUID NOT NULL REFERENCES tenants(id) ON DELETE CASCADE,
    employee_id UUID NOT NULL REFERENCES hr_employees(id) ON DELETE CASCADE,
    reference_month DATE NOT NULL, -- Primeiro dia do mês
    base_salary DECIMAL(12, 2) NOT NULL,
    overtime_hours DECIMAL(5, 2) DEFAULT 0,
    overtime_amount DECIMAL(10, 2) DEFAULT 0,
    bonuses DECIMAL(10, 2) DEFAULT 0,
    benefits DECIMAL(10, 2) DEFAULT 0,
    gross_salary DECIMAL(12, 2) NOT NULL,
    inss_deduction DECIMAL(10, 2) DEFAULT 0,
    irrf_deduction DECIMAL(10, 2) DEFAULT 0,
    other_deductions DECIMAL(10, 2) DEFAULT 0,
    total_deductions DECIMAL(10, 2) NOT NULL,
    net_salary DECIMAL(12, 2) NOT NULL,
    employer_inss DECIMAL(10, 2) DEFAULT 0,
    employer_fgts DECIMAL(10, 2) DEFAULT 0,
    payment_date DATE,
    status payment_status DEFAULT 'pending',
    created_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP
);

CREATE INDEX idx_payroll_tenant ON hr_payroll(tenant_id);
CREATE INDEX idx_payroll_employee ON hr_payroll(employee_id);
CREATE INDEX idx_payroll_month ON hr_payroll(reference_month);

-- Ponto Eletrônico
CREATE TABLE hr_attendance (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    tenant_id UUID NOT NULL REFERENCES tenants(id) ON DELETE CASCADE,
    employee_id UUID NOT NULL REFERENCES hr_employees(id) ON DELETE CASCADE,
    date DATE NOT NULL,
    check_in_1 TIME,
    check_out_1 TIME,
    check_in_2 TIME,
    check_out_2 TIME,
    total_hours DECIMAL(4, 2),
    overtime_hours DECIMAL(4, 2) DEFAULT 0,
    status VARCHAR(20) DEFAULT 'present', -- present, absent, on_leave, holiday
    notes TEXT,
    created_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP,
    UNIQUE(tenant_id, employee_id, date)
);

CREATE INDEX idx_attendance_tenant ON hr_attendance(tenant_id);
CREATE INDEX idx_attendance_employee ON hr_attendance(employee_id);
CREATE INDEX idx_attendance_date ON hr_attendance(date);

-- Avaliações de Desempenho
CREATE TABLE hr_performance_reviews (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    tenant_id UUID NOT NULL REFERENCES tenants(id) ON DELETE CASCADE,
    employee_id UUID NOT NULL REFERENCES hr_employees(id) ON DELETE CASCADE,
    reviewer_id UUID NOT NULL REFERENCES users(id),
    period VARCHAR(20) NOT NULL, -- 2024-H1, 2024-Q3
    review_type VARCHAR(20) NOT NULL, -- annual, quarterly, 360
    competencies JSONB NOT NULL, -- [{name: "Liderança", score: 4.5, weight: 25}]
    overall_score DECIMAL(3, 2),
    strengths TEXT,
    areas_for_improvement TEXT,
    goals_next_period TEXT,
    status VARCHAR(20) DEFAULT 'draft', -- draft, completed, approved
    completed_at TIMESTAMP WITH TIME ZONE,
    created_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP
);

CREATE INDEX idx_reviews_tenant ON hr_performance_reviews(tenant_id);
CREATE INDEX idx_reviews_employee ON hr_performance_reviews(employee_id);

-- ============================================================================
-- SUPPLY CHAIN & ESTOQUE
-- ============================================================================

CREATE TABLE inventory_products (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    tenant_id UUID NOT NULL REFERENCES tenants(id) ON DELETE CASCADE,
    sku VARCHAR(100) NOT NULL,
    name VARCHAR(255) NOT NULL,
    description TEXT,
    category VARCHAR(100),
    unit_of_measure VARCHAR(20), -- un, kg, l, m
    unit_cost DECIMAL(12, 2),
    sale_price DECIMAL(12, 2),
    current_stock INTEGER DEFAULT 0,
    min_stock INTEGER DEFAULT 0,
    max_stock INTEGER,
    reorder_point INTEGER,
    ncm VARCHAR(8), -- Código fiscal
    barcode VARCHAR(50),
    is_active BOOLEAN DEFAULT true,
    custom_fields JSONB DEFAULT '{}',
    created_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP,
    UNIQUE(tenant_id, sku)
);

CREATE INDEX idx_products_tenant ON inventory_products(tenant_id);
CREATE INDEX idx_products_sku ON inventory_products(sku);
CREATE INDEX idx_products_category ON inventory_products(category);

-- Movimentações de Estoque
CREATE TYPE stock_movement_type AS ENUM ('in', 'out', 'transfer', 'adjustment', 'return');

CREATE TABLE inventory_movements (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    tenant_id UUID NOT NULL REFERENCES tenants(id) ON DELETE CASCADE,
    product_id UUID NOT NULL REFERENCES inventory_products(id),
    type stock_movement_type NOT NULL,
    quantity INTEGER NOT NULL,
    unit_cost DECIMAL(12, 2),
    total_cost DECIMAL(12, 2),
    from_location VARCHAR(100),
    to_location VARCHAR(100),
    reference_id UUID, -- Pedido, venda, etc
    notes TEXT,
    created_by UUID REFERENCES users(id),
    created_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP
);

CREATE INDEX idx_movements_tenant ON inventory_movements(tenant_id);
CREATE INDEX idx_movements_product ON inventory_movements(product_id);
CREATE INDEX idx_movements_date ON inventory_movements(created_at);

-- Fornecedores
CREATE TABLE suppliers (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    tenant_id UUID NOT NULL REFERENCES tenants(id) ON DELETE CASCADE,
    name VARCHAR(255) NOT NULL,
    cnpj VARCHAR(18),
    email VARCHAR(255),
    phone VARCHAR(20),
    address JSONB,
    payment_terms VARCHAR(50), -- 30 dias, à vista, etc
    rating DECIMAL(2, 1) CHECK (rating >= 0 AND rating <= 5),
    is_active BOOLEAN DEFAULT true,
    created_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP
);

CREATE INDEX idx_suppliers_tenant ON suppliers(tenant_id);

-- ============================================================================
-- ANALYTICS & AUDITORIA
-- ============================================================================

-- Audit Log (Imutável)
CREATE TABLE audit_logs (
    id BIGSERIAL PRIMARY KEY,
    tenant_id UUID NOT NULL,
    user_id UUID,
    action VARCHAR(50) NOT NULL, -- create, update, delete, login, export
    resource_type VARCHAR(50) NOT NULL, -- lead, invoice, employee
    resource_id UUID,
    old_values JSONB,
    new_values JSONB,
    ip_address INET,
    user_agent TEXT,
    created_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP
);

CREATE INDEX idx_audit_tenant ON audit_logs(tenant_id);
CREATE INDEX idx_audit_resource ON audit_logs(resource_type, resource_id);
CREATE INDEX idx_audit_date ON audit_logs(created_at);

-- Webhooks
CREATE TABLE webhooks (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    tenant_id UUID NOT NULL REFERENCES tenants(id) ON DELETE CASCADE,
    url TEXT NOT NULL,
    events TEXT[] NOT NULL, -- {lead.created, invoice.paid}
    secret VARCHAR(255) NOT NULL,
    is_active BOOLEAN DEFAULT true,
    last_triggered_at TIMESTAMP WITH TIME ZONE,
    failure_count INTEGER DEFAULT 0,
    created_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP
);

CREATE INDEX idx_webhooks_tenant ON webhooks(tenant_id);

-- ============================================================================
-- FUNÇÕES E TRIGGERS
-- ============================================================================

-- Atualizar updated_at automaticamente
CREATE OR REPLACE FUNCTION update_updated_at_column()
RETURNS TRIGGER AS $$
BEGIN
    NEW.updated_at = CURRENT_TIMESTAMP;
    RETURN NEW;
END;
$$ language 'plpgsql';

-- Aplicar trigger em todas as tabelas relevantes
CREATE TRIGGER update_tenants_updated_at BEFORE UPDATE ON tenants
    FOR EACH ROW EXECUTE FUNCTION update_updated_at_column();

CREATE TRIGGER update_users_updated_at BEFORE UPDATE ON users
    FOR EACH ROW EXECUTE FUNCTION update_updated_at_column();

CREATE TRIGGER update_leads_updated_at BEFORE UPDATE ON crm_leads
    FOR EACH ROW EXECUTE FUNCTION update_updated_at_column();

-- Registrar mudança de estágio do lead
CREATE OR REPLACE FUNCTION log_lead_stage_change()
RETURNS TRIGGER AS $$
BEGIN
    IF OLD.stage IS DISTINCT FROM NEW.stage THEN
        INSERT INTO crm_lead_stage_history (lead_id, from_stage, to_stage, changed_by)
        VALUES (NEW.id, OLD.stage, NEW.stage, NEW.owner_id);
    END IF;
    RETURN NEW;
END;
$$ LANGUAGE plpgsql;

CREATE TRIGGER trigger_lead_stage_change
AFTER UPDATE ON crm_leads
FOR EACH ROW
EXECUTE FUNCTION log_lead_stage_change();

-- ============================================================================
-- VIEWS MATERIALIZADAS (Performance)
-- ============================================================================

-- Dashboard de vendas
CREATE MATERIALIZED VIEW mv_sales_dashboard AS
SELECT 
    tenant_id,
    DATE_TRUNC('month', created_at) as month,
    COUNT(*) as total_leads,
    COUNT(*) FILTER (WHERE stage = 'won') as won_deals,
    SUM(value) FILTER (WHERE stage = 'won') as total_revenue,
    AVG(score) as avg_score
FROM crm_leads
GROUP BY tenant_id, DATE_TRUNC('month', created_at);

CREATE UNIQUE INDEX ON mv_sales_dashboard (tenant_id, month);

-- Refresh automático (executar via cron)
-- REFRESH MATERIALIZED VIEW CONCURRENTLY mv_sales_dashboard;

-- ============================================================================
-- DADOS INICIAIS (SEED)
-- ============================================================================

-- Roles padrão
INSERT INTO roles (id, tenant_id, name, permissions, is_system) VALUES
    (uuid_generate_v4(), uuid_generate_v4(), 'Admin', '["*"]', true),
    (uuid_generate_v4(), uuid_generate_v4(), 'Sales Manager', '["crm.*", "analytics.view"]', true),
    (uuid_generate_v4(), uuid_generate_v4(), 'Sales Rep', '["crm.leads.*", "crm.activities.*"]', true),
    (uuid_generate_v4(), uuid_generate_v4(), 'Finance Manager', '["finance.*"]', true),
    (uuid_generate_v4(), uuid_generate_v4(), 'HR Manager', '["hr.*"]', true)
ON CONFLICT DO NOTHING;

-- ============================================================================
-- FIM DO SCHEMA
-- ============================================================================
