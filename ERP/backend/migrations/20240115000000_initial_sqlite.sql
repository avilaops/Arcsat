-- SQLite Migration: Initial Schema
-- Description: Schema inicial do ERP/CRM Faria Lima (SQLite)
-- Created: 2024-01-15

-- =============================================================================
-- CORE TABLES
-- =============================================================================

-- Tenants
CREATE TABLE IF NOT EXISTS tenants (
    id TEXT PRIMARY KEY,
    name TEXT NOT NULL,
    domain TEXT UNIQUE NOT NULL,
    status TEXT NOT NULL DEFAULT 'active',
    plan TEXT NOT NULL DEFAULT 'startup',
    max_users INTEGER NOT NULL DEFAULT 10,
    storage_limit_gb INTEGER NOT NULL DEFAULT 100,
    settings TEXT DEFAULT '{}',
    created_at TEXT DEFAULT CURRENT_TIMESTAMP,
    updated_at TEXT DEFAULT CURRENT_TIMESTAMP
);

CREATE INDEX IF NOT EXISTS idx_tenants_domain ON tenants(domain);
CREATE INDEX IF NOT EXISTS idx_tenants_status ON tenants(status);

-- Users
CREATE TABLE IF NOT EXISTS users (
    id TEXT PRIMARY KEY,
    tenant_id TEXT NOT NULL,
    email TEXT NOT NULL,
    password_hash TEXT,
    name TEXT NOT NULL,
    avatar_url TEXT,
    phone TEXT,
    language TEXT DEFAULT 'pt-BR',
    timezone TEXT DEFAULT 'America/Sao_Paulo',
    status TEXT NOT NULL DEFAULT 'active',
    last_login_at TEXT,
    created_at TEXT DEFAULT CURRENT_TIMESTAMP,
    updated_at TEXT DEFAULT CURRENT_TIMESTAMP,
    FOREIGN KEY (tenant_id) REFERENCES tenants(id) ON DELETE CASCADE,
    UNIQUE(tenant_id, email)
);

CREATE INDEX IF NOT EXISTS idx_users_tenant ON users(tenant_id);
CREATE INDEX IF NOT EXISTS idx_users_email ON users(email);

-- User Roles
CREATE TABLE IF NOT EXISTS user_roles (
    id TEXT PRIMARY KEY,
    user_id TEXT NOT NULL,
    role TEXT NOT NULL,
    created_at TEXT DEFAULT CURRENT_TIMESTAMP,
    FOREIGN KEY (user_id) REFERENCES users(id) ON DELETE CASCADE
);

CREATE INDEX IF NOT EXISTS idx_user_roles_user ON user_roles(user_id);

-- User Permissions
CREATE TABLE IF NOT EXISTS user_permissions (
    id TEXT PRIMARY KEY,
    user_id TEXT NOT NULL,
    permission TEXT NOT NULL,
    created_at TEXT DEFAULT CURRENT_TIMESTAMP,
    FOREIGN KEY (user_id) REFERENCES users(id) ON DELETE CASCADE
);

CREATE INDEX IF NOT EXISTS idx_user_permissions_user ON user_permissions(user_id);

-- =============================================================================
-- CRM TABLES
-- =============================================================================

-- Leads (usando TEXT para enums)
CREATE TABLE IF NOT EXISTS crm_leads (
    id TEXT PRIMARY KEY,
    tenant_id TEXT NOT NULL,
    name TEXT NOT NULL,
    company TEXT,
    email TEXT NOT NULL,
    phone TEXT,
    source TEXT NOT NULL, -- website, linkedin, referral, coldcall, event, partner, other
    stage TEXT NOT NULL, -- new, contacted, qualification, proposal, negotiation, won, lost
    score INTEGER DEFAULT 0,
    value REAL DEFAULT 0,
    probability INTEGER DEFAULT 0,
    owner_id TEXT NOT NULL,
    expected_close_date TEXT,
    actual_close_date TEXT,
    lost_reason TEXT,
    custom_fields TEXT DEFAULT '{}',
    created_at TEXT DEFAULT CURRENT_TIMESTAMP,
    updated_at TEXT DEFAULT CURRENT_TIMESTAMP,
    FOREIGN KEY (tenant_id) REFERENCES tenants(id) ON DELETE CASCADE,
    FOREIGN KEY (owner_id) REFERENCES users(id)
);

CREATE INDEX IF NOT EXISTS idx_crm_leads_tenant ON crm_leads(tenant_id);
CREATE INDEX IF NOT EXISTS idx_crm_leads_owner ON crm_leads(owner_id);
CREATE INDEX IF NOT EXISTS idx_crm_leads_stage ON crm_leads(stage);
CREATE INDEX IF NOT EXISTS idx_crm_leads_source ON crm_leads(source);

-- Lead Stage History
CREATE TABLE IF NOT EXISTS crm_lead_stage_history (
    id TEXT PRIMARY KEY,
    lead_id TEXT NOT NULL,
    old_stage TEXT,
    new_stage TEXT NOT NULL,
    changed_by TEXT NOT NULL,
    notes TEXT,
    created_at TEXT DEFAULT CURRENT_TIMESTAMP,
    FOREIGN KEY (lead_id) REFERENCES crm_leads(id) ON DELETE CASCADE,
    FOREIGN KEY (changed_by) REFERENCES users(id)
);

CREATE INDEX IF NOT EXISTS idx_lead_history_lead ON crm_lead_stage_history(lead_id);

-- Activities
CREATE TABLE IF NOT EXISTS crm_activities (
    id TEXT PRIMARY KEY,
    tenant_id TEXT NOT NULL,
    lead_id TEXT,
    account_id TEXT,
    contact_id TEXT,
    type TEXT NOT NULL, -- task, call, meeting, email, note
    status TEXT NOT NULL, -- scheduled, completed, canceled, overdue
    title TEXT NOT NULL,
    description TEXT,
    assigned_to TEXT NOT NULL,
    due_date TEXT,
    completed_at TEXT,
    custom_fields TEXT DEFAULT '{}',
    created_at TEXT DEFAULT CURRENT_TIMESTAMP,
    updated_at TEXT DEFAULT CURRENT_TIMESTAMP,
    FOREIGN KEY (tenant_id) REFERENCES tenants(id) ON DELETE CASCADE,
    FOREIGN KEY (assigned_to) REFERENCES users(id)
);

CREATE INDEX IF NOT EXISTS idx_activities_tenant ON crm_activities(tenant_id);
CREATE INDEX IF NOT EXISTS idx_activities_lead ON crm_activities(lead_id);
CREATE INDEX IF NOT EXISTS idx_activities_assigned ON crm_activities(assigned_to);
CREATE INDEX IF NOT EXISTS idx_activities_status ON crm_activities(status);

-- Accounts
CREATE TABLE IF NOT EXISTS crm_accounts (
    id TEXT PRIMARY KEY,
    tenant_id TEXT NOT NULL,
    name TEXT NOT NULL,
    industry TEXT,
    website TEXT,
    phone TEXT,
    billing_address TEXT,
    shipping_address TEXT,
    annual_revenue REAL,
    employee_count INTEGER,
    owner_id TEXT NOT NULL,
    health_score INTEGER DEFAULT 50,
    custom_fields TEXT DEFAULT '{}',
    created_at TEXT DEFAULT CURRENT_TIMESTAMP,
    updated_at TEXT DEFAULT CURRENT_TIMESTAMP,
    FOREIGN KEY (tenant_id) REFERENCES tenants(id) ON DELETE CASCADE,
    FOREIGN KEY (owner_id) REFERENCES users(id)
);

CREATE INDEX IF NOT EXISTS idx_accounts_tenant ON crm_accounts(tenant_id);
CREATE INDEX IF NOT EXISTS idx_accounts_owner ON crm_accounts(owner_id);

-- Contacts
CREATE TABLE IF NOT EXISTS crm_contacts (
    id TEXT PRIMARY KEY,
    tenant_id TEXT NOT NULL,
    account_id TEXT,
    first_name TEXT NOT NULL,
    last_name TEXT NOT NULL,
    email TEXT,
    phone TEXT,
    mobile TEXT,
    title TEXT,
    department TEXT,
    linkedin_url TEXT,
    is_primary INTEGER DEFAULT 0,
    custom_fields TEXT DEFAULT '{}',
    created_at TEXT DEFAULT CURRENT_TIMESTAMP,
    updated_at TEXT DEFAULT CURRENT_TIMESTAMP,
    FOREIGN KEY (tenant_id) REFERENCES tenants(id) ON DELETE CASCADE,
    FOREIGN KEY (account_id) REFERENCES crm_accounts(id) ON DELETE CASCADE
);

CREATE INDEX IF NOT EXISTS idx_contacts_tenant ON crm_contacts(tenant_id);
CREATE INDEX IF NOT EXISTS idx_contacts_account ON crm_contacts(account_id);
CREATE INDEX IF NOT EXISTS idx_contacts_email ON crm_contacts(email);

-- =============================================================================
-- FINANCE TABLES
-- =============================================================================

-- Accounts Payable
CREATE TABLE IF NOT EXISTS finance_accounts_payable (
    id TEXT PRIMARY KEY,
    tenant_id TEXT NOT NULL,
    vendor_name TEXT NOT NULL,
    invoice_number TEXT,
    description TEXT,
    amount REAL NOT NULL,
    due_date TEXT NOT NULL,
    payment_date TEXT,
    status TEXT NOT NULL, -- pending, approved, paid, canceled, overdue
    payment_method TEXT, -- pix, banktransfer, bankslip, creditcard, cash, other
    notes TEXT,
    created_at TEXT DEFAULT CURRENT_TIMESTAMP,
    updated_at TEXT DEFAULT CURRENT_TIMESTAMP,
    FOREIGN KEY (tenant_id) REFERENCES tenants(id) ON DELETE CASCADE
);

CREATE INDEX IF NOT EXISTS idx_payable_tenant ON finance_accounts_payable(tenant_id);
CREATE INDEX IF NOT EXISTS idx_payable_status ON finance_accounts_payable(status);

-- Accounts Receivable
CREATE TABLE IF NOT EXISTS finance_accounts_receivable (
    id TEXT PRIMARY KEY,
    tenant_id TEXT NOT NULL,
    customer_name TEXT NOT NULL,
    invoice_number TEXT,
    description TEXT,
    amount REAL NOT NULL,
    due_date TEXT NOT NULL,
    payment_date TEXT,
    status TEXT NOT NULL, -- pending, approved, paid, canceled, overdue
    payment_method TEXT,
    notes TEXT,
    created_at TEXT DEFAULT CURRENT_TIMESTAMP,
    updated_at TEXT DEFAULT CURRENT_TIMESTAMP,
    FOREIGN KEY (tenant_id) REFERENCES tenants(id) ON DELETE CASCADE
);

CREATE INDEX IF NOT EXISTS idx_receivable_tenant ON finance_accounts_receivable(tenant_id);
CREATE INDEX IF NOT EXISTS idx_receivable_status ON finance_accounts_receivable(status);

-- =============================================================================
-- HR TABLES
-- =============================================================================

-- Employees
CREATE TABLE IF NOT EXISTS hr_employees (
    id TEXT PRIMARY KEY,
    tenant_id TEXT NOT NULL,
    user_id TEXT,
    employee_number TEXT UNIQUE NOT NULL,
    first_name TEXT NOT NULL,
    last_name TEXT NOT NULL,
    email TEXT NOT NULL,
    phone TEXT,
    department TEXT,
    position TEXT,
    employment_type TEXT NOT NULL, -- clt, pj, intern, contractor
    status TEXT NOT NULL, -- active, onleave, resigned, terminated
    hire_date TEXT NOT NULL,
    termination_date TEXT,
    salary REAL,
    manager_id TEXT,
    custom_fields TEXT DEFAULT '{}',
    created_at TEXT DEFAULT CURRENT_TIMESTAMP,
    updated_at TEXT DEFAULT CURRENT_TIMESTAMP,
    FOREIGN KEY (tenant_id) REFERENCES tenants(id) ON DELETE CASCADE,
    FOREIGN KEY (user_id) REFERENCES users(id),
    FOREIGN KEY (manager_id) REFERENCES hr_employees(id)
);

CREATE INDEX IF NOT EXISTS idx_employees_tenant ON hr_employees(tenant_id);
CREATE INDEX IF NOT EXISTS idx_employees_status ON hr_employees(status);
CREATE INDEX IF NOT EXISTS idx_employees_department ON hr_employees(department);

-- =============================================================================
-- WEBHOOKS
-- =============================================================================

CREATE TABLE IF NOT EXISTS webhooks (
    id TEXT PRIMARY KEY,
    tenant_id TEXT NOT NULL,
    name TEXT NOT NULL,
    url TEXT NOT NULL,
    secret TEXT,
    events TEXT NOT NULL, -- JSON array
    is_active INTEGER DEFAULT 1,
    last_triggered_at TEXT,
    failure_count INTEGER DEFAULT 0,
    created_at TEXT DEFAULT CURRENT_TIMESTAMP,
    updated_at TEXT DEFAULT CURRENT_TIMESTAMP,
    FOREIGN KEY (tenant_id) REFERENCES tenants(id) ON DELETE CASCADE
);

CREATE INDEX IF NOT EXISTS idx_webhooks_tenant ON webhooks(tenant_id);
CREATE INDEX IF NOT EXISTS idx_webhooks_active ON webhooks(is_active);
