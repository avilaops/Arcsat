-- Migration: 001_initial_schema
-- Description: Schema inicial do ERP/CRM Faria Lima
-- Created: 2024-01-15

-- =============================================================================
-- EXTENSIONS
-- =============================================================================

CREATE EXTENSION IF NOT EXISTS "uuid-ossp";
CREATE EXTENSION IF NOT EXISTS "pg_trgm"; -- Para full-text search

-- =============================================================================
-- ENUMS
-- =============================================================================

-- CRM
CREATE TYPE lead_stage AS ENUM ('new', 'contacted', 'qualification', 'proposal', 'negotiation', 'won', 'lost');
CREATE TYPE lead_source AS ENUM ('website', 'linkedin', 'referral', 'coldcall', 'event', 'partner', 'other');
CREATE TYPE activity_type AS ENUM ('task', 'call', 'meeting', 'email', 'note');
CREATE TYPE activity_status AS ENUM ('scheduled', 'completed', 'canceled', 'overdue');

-- Finance
CREATE TYPE payment_status AS ENUM ('pending', 'approved', 'paid', 'canceled', 'overdue');
CREATE TYPE payment_method AS ENUM ('pix', 'banktransfer', 'bankslip', 'creditcard', 'cash', 'other');

-- HR
CREATE TYPE employment_type AS ENUM ('clt', 'pj', 'intern', 'contractor');
CREATE TYPE employee_status AS ENUM ('active', 'onleave', 'resigned', 'terminated');

-- =============================================================================
-- CORE TABLES
-- =============================================================================

-- Tenants
CREATE TABLE tenants (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    name VARCHAR(255) NOT NULL,
    domain VARCHAR(100) UNIQUE NOT NULL,
    status VARCHAR(50) NOT NULL DEFAULT 'active',
    plan VARCHAR(50) NOT NULL DEFAULT 'startup',
    max_users INTEGER NOT NULL DEFAULT 10,
    storage_limit_gb INTEGER NOT NULL DEFAULT 100,
    settings JSONB DEFAULT '{}',
    created_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP
);

CREATE INDEX idx_tenants_domain ON tenants(domain);
CREATE INDEX idx_tenants_status ON tenants(status);

-- Users
CREATE TABLE users (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    tenant_id UUID NOT NULL REFERENCES tenants(id) ON DELETE CASCADE,
    email VARCHAR(255) NOT NULL,
    password_hash VARCHAR(255),
    name VARCHAR(255) NOT NULL,
    avatar_url TEXT,
    phone VARCHAR(20),
    language VARCHAR(10) DEFAULT 'pt-BR',
    timezone VARCHAR(50) DEFAULT 'America/Sao_Paulo',
    status VARCHAR(50) NOT NULL DEFAULT 'active',
    last_login_at TIMESTAMP WITH TIME ZONE,
    created_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP,
    UNIQUE(tenant_id, email)
);

CREATE INDEX idx_users_tenant ON users(tenant_id);
CREATE INDEX idx_users_email ON users(email);

-- User Roles
CREATE TABLE user_roles (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    role VARCHAR(50) NOT NULL,
    created_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP
);

CREATE INDEX idx_user_roles_user ON user_roles(user_id);

-- User Permissions
CREATE TABLE user_permissions (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    permission VARCHAR(100) NOT NULL,
    created_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP
);

CREATE INDEX idx_user_permissions_user ON user_permissions(user_id);

-- =============================================================================
-- CRM TABLES
-- =============================================================================

-- Leads
CREATE TABLE crm_leads (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    tenant_id UUID NOT NULL REFERENCES tenants(id) ON DELETE CASCADE,
    name VARCHAR(255) NOT NULL,
    company VARCHAR(255),
    email VARCHAR(255) NOT NULL,
    phone VARCHAR(20),
    source lead_source NOT NULL,
    stage lead_stage NOT NULL DEFAULT 'new',
    score INTEGER NOT NULL DEFAULT 50 CHECK (score >= 0 AND score <= 100),
    value DECIMAL(15, 2) NOT NULL DEFAULT 0,
    probability INTEGER NOT NULL DEFAULT 10 CHECK (probability >= 0 AND probability <= 100),
    owner_id UUID NOT NULL REFERENCES users(id),
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
CREATE INDEX idx_leads_email ON crm_leads(email);

-- Lead Stage History
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

-- Accounts
CREATE TABLE crm_accounts (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    tenant_id UUID NOT NULL REFERENCES tenants(id) ON DELETE CASCADE,
    name VARCHAR(255) NOT NULL,
    cnpj VARCHAR(18),
    industry VARCHAR(100),
    employees_count INTEGER,
    annual_revenue DECIMAL(15, 2),
    website TEXT,
    address JSONB,
    account_manager_id UUID REFERENCES users(id),
    health_score INTEGER CHECK (health_score >= 0 AND health_score <= 100),
    status VARCHAR(50) NOT NULL DEFAULT 'active',
    custom_fields JSONB DEFAULT '{}',
    created_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP
);

CREATE INDEX idx_accounts_tenant ON crm_accounts(tenant_id);
CREATE INDEX idx_accounts_cnpj ON crm_accounts(cnpj);
CREATE INDEX idx_accounts_manager ON crm_accounts(account_manager_id);
CREATE INDEX idx_accounts_health ON crm_accounts(health_score);

-- Contacts
CREATE TABLE crm_contacts (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    tenant_id UUID NOT NULL REFERENCES tenants(id) ON DELETE CASCADE,
    account_id UUID REFERENCES crm_accounts(id) ON DELETE CASCADE,
    name VARCHAR(255) NOT NULL,
    email VARCHAR(255),
    phone VARCHAR(20),
    position VARCHAR(100),
    department VARCHAR(100),
    is_decision_maker BOOLEAN DEFAULT FALSE,
    linkedin_url TEXT,
    custom_fields JSONB DEFAULT '{}',
    created_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP
);

CREATE INDEX idx_contacts_tenant ON crm_contacts(tenant_id);
CREATE INDEX idx_contacts_account ON crm_contacts(account_id);
CREATE INDEX idx_contacts_email ON crm_contacts(email);

-- Activities
CREATE TABLE crm_activities (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    tenant_id UUID NOT NULL REFERENCES tenants(id) ON DELETE CASCADE,
    type activity_type NOT NULL,
    subject VARCHAR(255) NOT NULL,
    description TEXT,
    status activity_status NOT NULL DEFAULT 'scheduled',
    scheduled_at TIMESTAMP WITH TIME ZONE,
    completed_at TIMESTAMP WITH TIME ZONE,
    duration_minutes INTEGER,
    lead_id UUID REFERENCES crm_leads(id) ON DELETE CASCADE,
    account_id UUID REFERENCES crm_accounts(id) ON DELETE CASCADE,
    contact_id UUID REFERENCES crm_contacts(id) ON DELETE CASCADE,
    owner_id UUID REFERENCES users(id),
    attendees JSONB DEFAULT '[]',
    custom_fields JSONB DEFAULT '{}',
    created_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP
);

CREATE INDEX idx_activities_tenant ON crm_activities(tenant_id);
CREATE INDEX idx_activities_type ON crm_activities(type);
CREATE INDEX idx_activities_status ON crm_activities(status);
CREATE INDEX idx_activities_lead ON crm_activities(lead_id);
CREATE INDEX idx_activities_scheduled ON crm_activities(scheduled_at);

-- =============================================================================
-- FINANCE TABLES
-- =============================================================================

-- Accounts Payable
CREATE TABLE finance_accounts_payable (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    tenant_id UUID NOT NULL REFERENCES tenants(id) ON DELETE CASCADE,
    supplier_id UUID,
    invoice_number VARCHAR(100),
    description TEXT NOT NULL,
    amount DECIMAL(15, 2) NOT NULL,
    due_date DATE NOT NULL,
    payment_date DATE,
    payment_method payment_method,
    status payment_status NOT NULL DEFAULT 'pending',
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
CREATE INDEX idx_ap_status ON finance_accounts_payable(status);
CREATE INDEX idx_ap_due_date ON finance_accounts_payable(due_date);

-- Accounts Receivable
CREATE TABLE finance_accounts_receivable (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    tenant_id UUID NOT NULL REFERENCES tenants(id) ON DELETE CASCADE,
    customer_id UUID,
    invoice_number VARCHAR(100),
    description TEXT NOT NULL,
    amount DECIMAL(15, 2) NOT NULL,
    due_date DATE NOT NULL,
    payment_date DATE,
    payment_method payment_method,
    status payment_status NOT NULL DEFAULT 'pending',
    notes TEXT,
    nfe_key VARCHAR(44),
    nfe_url TEXT,
    created_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP
);

CREATE INDEX idx_ar_tenant ON finance_accounts_receivable(tenant_id);
CREATE INDEX idx_ar_status ON finance_accounts_receivable(status);
CREATE INDEX idx_ar_due_date ON finance_accounts_receivable(due_date);

-- =============================================================================
-- HR TABLES
-- =============================================================================

-- Employees
CREATE TABLE hr_employees (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    tenant_id UUID NOT NULL REFERENCES tenants(id) ON DELETE CASCADE,
    user_id UUID REFERENCES users(id),
    full_name VARCHAR(255) NOT NULL,
    cpf VARCHAR(14) NOT NULL,
    rg VARCHAR(20),
    birth_date DATE,
    email VARCHAR(255),
    phone VARCHAR(20),
    address JSONB,
    employment_type employment_type NOT NULL,
    status employee_status NOT NULL DEFAULT 'active',
    department VARCHAR(100),
    position VARCHAR(100),
    manager_id UUID REFERENCES hr_employees(id),
    admission_date DATE NOT NULL,
    resignation_date DATE,
    base_salary DECIMAL(15, 2),
    benefits JSONB DEFAULT '{}',
    bank_info JSONB,
    performance_score DECIMAL(3, 2),
    custom_fields JSONB DEFAULT '{}',
    created_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP,
    UNIQUE(tenant_id, cpf)
);

CREATE INDEX idx_employees_tenant ON hr_employees(tenant_id);
CREATE INDEX idx_employees_status ON hr_employees(status);
CREATE INDEX idx_employees_department ON hr_employees(department);
CREATE INDEX idx_employees_cpf ON hr_employees(cpf);

-- =============================================================================
-- SYSTEM TABLES
-- =============================================================================

-- Webhooks
CREATE TABLE webhooks (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    tenant_id UUID NOT NULL REFERENCES tenants(id) ON DELETE CASCADE,
    url TEXT NOT NULL,
    events TEXT[] NOT NULL,
    secret VARCHAR(255) NOT NULL,
    is_active BOOLEAN DEFAULT TRUE,
    last_triggered_at TIMESTAMP WITH TIME ZONE,
    failure_count INTEGER DEFAULT 0,
    created_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP
);

CREATE INDEX idx_webhooks_tenant ON webhooks(tenant_id);
CREATE INDEX idx_webhooks_active ON webhooks(is_active);

-- Audit Logs
CREATE TABLE audit_logs (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    tenant_id UUID NOT NULL REFERENCES tenants(id) ON DELETE CASCADE,
    user_id UUID REFERENCES users(id),
    action VARCHAR(100) NOT NULL,
    resource_type VARCHAR(100) NOT NULL,
    resource_id UUID,
    changes JSONB,
    ip_address INET,
    user_agent TEXT,
    created_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP
);

CREATE INDEX idx_audit_tenant ON audit_logs(tenant_id);
CREATE INDEX idx_audit_user ON audit_logs(user_id);
CREATE INDEX idx_audit_resource ON audit_logs(resource_type, resource_id);
CREATE INDEX idx_audit_created ON audit_logs(created_at);

-- =============================================================================
-- TRIGGERS
-- =============================================================================

-- Updated_at trigger function
CREATE OR REPLACE FUNCTION update_updated_at_column()
RETURNS TRIGGER AS $$
BEGIN
    NEW.updated_at = CURRENT_TIMESTAMP;
    RETURN NEW;
END;
$$ LANGUAGE plpgsql;

-- Apply trigger to all tables with updated_at
CREATE TRIGGER update_tenants_updated_at BEFORE UPDATE ON tenants FOR EACH ROW EXECUTE FUNCTION update_updated_at_column();
CREATE TRIGGER update_users_updated_at BEFORE UPDATE ON users FOR EACH ROW EXECUTE FUNCTION update_updated_at_column();
CREATE TRIGGER update_leads_updated_at BEFORE UPDATE ON crm_leads FOR EACH ROW EXECUTE FUNCTION update_updated_at_column();
CREATE TRIGGER update_accounts_updated_at BEFORE UPDATE ON crm_accounts FOR EACH ROW EXECUTE FUNCTION update_updated_at_column();
CREATE TRIGGER update_contacts_updated_at BEFORE UPDATE ON crm_contacts FOR EACH ROW EXECUTE FUNCTION update_updated_at_column();
CREATE TRIGGER update_ap_updated_at BEFORE UPDATE ON finance_accounts_payable FOR EACH ROW EXECUTE FUNCTION update_updated_at_column();
CREATE TRIGGER update_ar_updated_at BEFORE UPDATE ON finance_accounts_receivable FOR EACH ROW EXECUTE FUNCTION update_updated_at_column();
CREATE TRIGGER update_employees_updated_at BEFORE UPDATE ON hr_employees FOR EACH ROW EXECUTE FUNCTION update_updated_at_column();

-- =============================================================================
-- SEED DATA (Development only)
-- =============================================================================

-- Default tenant
INSERT INTO tenants (id, name, domain, plan) 
VALUES 
    ('00000000-0000-0000-0000-000000000001', 'Default Tenant', 'default', 'enterprise');

-- Default admin user
INSERT INTO users (id, tenant_id, email, name, status)
VALUES
    ('00000000-0000-0000-0000-000000000001', '00000000-0000-0000-0000-000000000001', 'admin@erp.com', 'Admin User', 'active');

INSERT INTO user_roles (user_id, role)
VALUES
    ('00000000-0000-0000-0000-000000000001', 'admin');

INSERT INTO user_permissions (user_id, permission)
VALUES
    ('00000000-0000-0000-0000-000000000001', '*');
