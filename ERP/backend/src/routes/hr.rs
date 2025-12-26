// routes/hr.rs - Módulo de Recursos Humanos Enterprise

use crate::{
    auth::Claims,
    db::DbPool,
    error::{AppError, Result},
    models::*,
};
use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    response::Json,
    routing::{get, post, patch, delete},
    Router,
};
use rust_decimal::Decimal;
use std::sync::Arc;
use uuid::Uuid;
use validator::Validate;
use chrono::{Datelike, NaiveDate, Utc};

pub fn routes(pool: DbPool) -> Router {
    Router::new()
        // Employees
        .route("/employees", post(create_employee).get(list_employees))
        .route("/employees/:id", get(get_employee).patch(update_employee).delete(delete_employee))
        .route("/employees/:id/terminate", post(terminate_employee))
        
        // Payroll
        .route("/payroll/calculate", post(calculate_payroll))
        .route("/payroll/months/:month", get(get_payroll_by_month))
        .route("/payroll/:id", get(get_payroll))
        
        // Attendance
        .route("/attendance", post(register_attendance).get(list_attendance))
        .route("/attendance/:employee_id/:date", get(get_attendance))
        .route("/attendance/report", get(attendance_report))
        
        // Performance Reviews
        .route("/performance-reviews", post(create_performance_review).get(list_performance_reviews))
        .route("/performance-reviews/:id", get(get_performance_review).patch(update_performance_review))
        
        // Analytics
        .route("/analytics/turnover", get(turnover_analytics))
        .route("/analytics/headcount", get(headcount_analytics))
        
        .with_state(Arc::new(pool))
}

// ============================================================================
// EMPLOYEES HANDLERS
// ============================================================================

#[derive(Debug, serde::Deserialize, Validate)]
struct CreateEmployeeRequest {
    #[validate(length(min = 3, max = 255))]
    full_name: String,
    
    #[validate(length(min = 11, max = 14))]
    cpf: String,
    
    #[validate(length(max = 20))]
    rg: Option<String>,
    
    birth_date: Option<NaiveDate>,
    
    #[validate(email)]
    email: Option<String>,
    
    #[validate(length(max = 20))]
    phone: Option<String>,
    
    address: Option<serde_json::Value>,
    employment_type: EmploymentType,
    department: Option<String>,
    position: Option<String>,
    manager_id: Option<Uuid>,
    admission_date: NaiveDate,
    base_salary: Option<f64>,
    benefits: Option<serde_json::Value>,
    bank_info: Option<serde_json::Value>,
}

async fn create_employee(
    claims: Claims,
    State(pool): State<Arc<DbPool>>,
    Json(payload): Json<CreateEmployeeRequest>,
) -> Result<(StatusCode, Json<Employee>)> {
    payload.validate()
        .map_err(|e| AppError::validation_error(e.to_string()))?;

    let id = Uuid::new_v4();
    let tenant_id = Uuid::parse_str(&claims.tenant_id)?;
    
    // Validar CPF
    validate_cpf(&payload.cpf)?;

    let employee = sqlx::query_as!(
        Employee,
        r#"
        INSERT INTO hr_employees
            (id, tenant_id, full_name, cpf, rg, birth_date, email, phone, address,
             employment_type, status, department, position, manager_id, admission_date,
             base_salary, benefits, bank_info, custom_fields)
        VALUES
            ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10::employment_type, $11::employee_status,
             $12, $13, $14, $15, $16, $17, $18, '{}')
        RETURNING
            id, tenant_id, user_id, full_name, cpf, rg, birth_date, email, phone,
            address as "address: Option<serde_json::Value>",
            employment_type as "employment_type: EmploymentType",
            status as "status: EmployeeStatus",
            department, position, manager_id, admission_date, resignation_date,
            base_salary, benefits as "benefits: serde_json::Value",
            bank_info as "bank_info: Option<serde_json::Value>",
            performance_score,
            custom_fields as "custom_fields: serde_json::Value",
            created_at, updated_at
        "#,
        id,
        tenant_id,
        payload.full_name,
        payload.cpf,
        payload.rg,
        payload.birth_date,
        payload.email,
        payload.phone,
        payload.address,
        payload.employment_type as EmploymentType,
        EmployeeStatus::Active as EmployeeStatus,
        payload.department,
        payload.position,
        payload.manager_id,
        payload.admission_date,
        payload.base_salary.map(|s| Decimal::from_f64_retain(s).unwrap()),
        payload.benefits.unwrap_or(serde_json::json!({})),
        payload.bank_info,
    )
    .fetch_one(pool.as_ref())
    .await?;

    tracing::info!("Employee created: {} - {}", id, payload.full_name);

    Ok((StatusCode::CREATED, Json(employee)))
}

#[derive(Debug, serde::Deserialize)]
struct ListEmployeesQuery {
    department: Option<String>,
    status: Option<EmployeeStatus>,
    employment_type: Option<EmploymentType>,
    page: Option<i32>,
    limit: Option<i32>,
}

async fn list_employees(
    claims: Claims,
    State(pool): State<Arc<DbPool>>,
    Query(query): Query<ListEmployeesQuery>,
) -> Result<Json<PaginatedResponse<Employee>>> {
    let tenant_id = Uuid::parse_str(&claims.tenant_id)?;
    
    let page = query.page.unwrap_or(1).max(1);
    let limit = query.limit.unwrap_or(50).min(100);
    let offset = (page - 1) * limit;

    let total_items = sqlx::query_scalar::<_, i64>(
        r#"
        SELECT COUNT(*)
        FROM hr_employees
        WHERE tenant_id = $1
          AND ($2::text IS NULL OR department = $2)
          AND ($3::employee_status IS NULL OR status = $3)
          AND ($4::employment_type IS NULL OR employment_type = $4)
        "#
    )
    .bind(tenant_id)
    .bind(query.department.as_ref())
    .bind(query.status.as_ref())
    .bind(query.employment_type.as_ref())
    .fetch_one(pool.as_ref())
    .await?;

    let employees = sqlx::query_as!(
        Employee,
        r#"
        SELECT
            id, tenant_id, user_id, full_name, cpf, rg, birth_date, email, phone,
            address as "address: Option<serde_json::Value>",
            employment_type as "employment_type: EmploymentType",
            status as "status: EmployeeStatus",
            department, position, manager_id, admission_date, resignation_date,
            base_salary, benefits as "benefits: serde_json::Value",
            bank_info as "bank_info: Option<serde_json::Value>",
            performance_score,
            custom_fields as "custom_fields: serde_json::Value",
            created_at, updated_at
        FROM hr_employees
        WHERE tenant_id = $1
          AND ($2::text IS NULL OR department = $2)
          AND ($3::employee_status IS NULL OR status = $3)
          AND ($4::employment_type IS NULL OR employment_type = $4)
        ORDER BY full_name
        LIMIT $5 OFFSET $6
        "#,
        tenant_id,
        query.department,
        query.status as Option<EmployeeStatus>,
        query.employment_type as Option<EmploymentType>,
        limit as i64,
        offset as i64,
    )
    .fetch_all(pool.as_ref())
    .await?;

    Ok(Json(PaginatedResponse::new(employees, page, limit, total_items)))
}

async fn get_employee(
    claims: Claims,
    State(pool): State<Arc<DbPool>>,
    Path(id): Path<Uuid>,
) -> Result<Json<Employee>> {
    let tenant_id = Uuid::parse_str(&claims.tenant_id)?;

    let employee = sqlx::query_as!(
        Employee,
        r#"
        SELECT
            id, tenant_id, user_id, full_name, cpf, rg, birth_date, email, phone,
            address as "address: Option<serde_json::Value>",
            employment_type as "employment_type: EmploymentType",
            status as "status: EmployeeStatus",
            department, position, manager_id, admission_date, resignation_date,
            base_salary, benefits as "benefits: serde_json::Value",
            bank_info as "bank_info: Option<serde_json::Value>",
            performance_score,
            custom_fields as "custom_fields: serde_json::Value",
            created_at, updated_at
        FROM hr_employees
        WHERE id = $1 AND tenant_id = $2
        "#,
        id,
        tenant_id
    )
    .fetch_optional(pool.as_ref())
    .await?
    .ok_or_else(|| AppError::not_found("Employee"))?;

    Ok(Json(employee))
}

async fn update_employee(
    claims: Claims,
    State(pool): State<Arc<DbPool>>,
    Path(id): Path<Uuid>,
    Json(payload): Json<CreateEmployeeRequest>,
) -> Result<Json<Employee>> {
    payload.validate()
        .map_err(|e| AppError::validation_error(e.to_string()))?;

    let tenant_id = Uuid::parse_str(&claims.tenant_id)?;

    let employee = sqlx::query_as!(
        Employee,
        r#"
        UPDATE hr_employees
        SET
            full_name = $1,
            rg = $2,
            birth_date = $3,
            email = $4,
            phone = $5,
            address = $6,
            department = $7,
            position = $8,
            manager_id = $9,
            base_salary = $10,
            benefits = $11,
            bank_info = $12,
            updated_at = NOW()
        WHERE id = $13 AND tenant_id = $14
        RETURNING
            id, tenant_id, user_id, full_name, cpf, rg, birth_date, email, phone,
            address as "address: Option<serde_json::Value>",
            employment_type as "employment_type: EmploymentType",
            status as "status: EmployeeStatus",
            department, position, manager_id, admission_date, resignation_date,
            base_salary, benefits as "benefits: serde_json::Value",
            bank_info as "bank_info: Option<serde_json::Value>",
            performance_score,
            custom_fields as "custom_fields: serde_json::Value",
            created_at, updated_at
        "#,
        payload.full_name,
        payload.rg,
        payload.birth_date,
        payload.email,
        payload.phone,
        payload.address,
        payload.department,
        payload.position,
        payload.manager_id,
        payload.base_salary.map(|s| Decimal::from_f64_retain(s).unwrap()),
        payload.benefits.unwrap_or(serde_json::json!({})),
        payload.bank_info,
        id,
        tenant_id
    )
    .fetch_optional(pool.as_ref())
    .await?
    .ok_or_else(|| AppError::not_found("Employee"))?;

    Ok(Json(employee))
}

async fn delete_employee(
    claims: Claims,
    State(pool): State<Arc<DbPool>>,
    Path(id): Path<Uuid>,
) -> Result<StatusCode> {
    let tenant_id = Uuid::parse_str(&claims.tenant_id)?;

    // Soft delete - marcar como terminado
    let result = sqlx::query!(
        r#"
        UPDATE hr_employees
        SET status = $1::employee_status, resignation_date = CURRENT_DATE, updated_at = NOW()
        WHERE id = $2 AND tenant_id = $3
        "#,
        EmployeeStatus::Terminated as EmployeeStatus,
        id,
        tenant_id
    )
    .execute(pool.as_ref())
    .await?;

    if result.rows_affected() == 0 {
        return Err(AppError::not_found("Employee"));
    }

    Ok(StatusCode::NO_CONTENT)
}

#[derive(Debug, serde::Deserialize)]
struct TerminateEmployeeRequest {
    resignation_date: NaiveDate,
    reason: Option<String>,
}

async fn terminate_employee(
    claims: Claims,
    State(pool): State<Arc<DbPool>>,
    Path(id): Path<Uuid>,
    Json(payload): Json<TerminateEmployeeRequest>,
) -> Result<Json<Employee>> {
    let tenant_id = Uuid::parse_str(&claims.tenant_id)?;

    let employee = sqlx::query_as!(
        Employee,
        r#"
        UPDATE hr_employees
        SET
            status = $1::employee_status,
            resignation_date = $2,
            updated_at = NOW()
        WHERE id = $3 AND tenant_id = $4
        RETURNING
            id, tenant_id, user_id, full_name, cpf, rg, birth_date, email, phone,
            address as "address: Option<serde_json::Value>",
            employment_type as "employment_type: EmploymentType",
            status as "status: EmployeeStatus",
            department, position, manager_id, admission_date, resignation_date,
            base_salary, benefits as "benefits: serde_json::Value",
            bank_info as "bank_info: Option<serde_json::Value>",
            performance_score,
            custom_fields as "custom_fields: serde_json::Value",
            created_at, updated_at
        "#,
        EmployeeStatus::Terminated as EmployeeStatus,
        payload.resignation_date,
        id,
        tenant_id
    )
    .fetch_optional(pool.as_ref())
    .await?
    .ok_or_else(|| AppError::not_found("Employee"))?;

    tracing::info!("Employee terminated: {} - {}", id, employee.full_name);

    Ok(Json(employee))
}

// ============================================================================
// PAYROLL
// ============================================================================

#[derive(Debug, serde::Deserialize)]
struct CalculatePayrollRequest {
    reference_month: String, // YYYY-MM
    employee_ids: Option<Vec<Uuid>>,
    include_benefits: bool,
    include_overtime: bool,
}

#[derive(Debug, serde::Serialize)]
struct PayrollCalculationResponse {
    reference_month: String,
    total_gross: f64,
    total_deductions: f64,
    total_net: f64,
    employer_charges: EmployerCharges,
    employees: Vec<PayrollCalculation>,
}

#[derive(Debug, serde::Serialize)]
struct EmployerCharges {
    inss: f64,
    fgts: f64,
    total: f64,
}

async fn calculate_payroll(
    claims: Claims,
    State(pool): State<Arc<DbPool>>,
    Json(payload): Json<CalculatePayrollRequest>,
) -> Result<Json<PayrollCalculationResponse>> {
    let tenant_id = Uuid::parse_str(&claims.tenant_id)?;

    // Buscar funcionários ativos
    let employees = if let Some(ids) = payload.employee_ids {
        sqlx::query_as!(
            Employee,
            r#"
            SELECT
                id, tenant_id, user_id, full_name, cpf, rg, birth_date, email, phone,
                address as "address: Option<serde_json::Value>",
                employment_type as "employment_type: EmploymentType",
                status as "status: EmployeeStatus",
                department, position, manager_id, admission_date, resignation_date,
                base_salary, benefits as "benefits: serde_json::Value",
                bank_info as "bank_info: Option<serde_json::Value>",
                performance_score,
                custom_fields as "custom_fields: serde_json::Value",
                created_at, updated_at
            FROM hr_employees
            WHERE tenant_id = $1 AND id = ANY($2) AND status = $3::employee_status
            "#,
            tenant_id,
            &ids,
            EmployeeStatus::Active as EmployeeStatus
        )
        .fetch_all(pool.as_ref())
        .await?
    } else {
        sqlx::query_as!(
            Employee,
            r#"
            SELECT
                id, tenant_id, user_id, full_name, cpf, rg, birth_date, email, phone,
                address as "address: Option<serde_json::Value>",
                employment_type as "employment_type: EmploymentType",
                status as "status: EmployeeStatus",
                department, position, manager_id, admission_date, resignation_date,
                base_salary, benefits as "benefits: serde_json::Value",
                bank_info as "bank_info: Option<serde_json::Value>",
                performance_score,
                custom_fields as "custom_fields: serde_json::Value",
                created_at, updated_at
            FROM hr_employees
            WHERE tenant_id = $1 AND status = $2::employee_status
            "#,
            tenant_id,
            EmployeeStatus::Active as EmployeeStatus
        )
        .fetch_all(pool.as_ref())
        .await?
    };

    let mut payroll_calculations = Vec::new();
    let mut total_gross = 0.0;
    let mut total_deductions_sum = 0.0;
    let mut total_net = 0.0;
    let mut employer_inss = 0.0;
    let mut employer_fgts = 0.0;

    for employee in employees {
        let base_salary = employee.base_salary
            .map(|s| s.to_f64().unwrap_or(0.0))
            .unwrap_or(0.0);

        // Calcular horas extras (buscar do ponto eletrônico)
        let overtime = if payload.include_overtime {
            calculate_overtime(&pool, employee.id, &payload.reference_month).await?
        } else {
            0.0
        };

        // Benefícios
        let benefits = if payload.include_benefits {
            extract_benefits_value(&employee.benefits)
        } else {
            0.0
        };

        let gross_salary = base_salary + overtime + benefits;

        // Calcular descontos
        let inss = calculate_inss(base_salary);
        let irrf = calculate_irrf(base_salary - inss);
        let health_plan = 450.0; // TODO: Buscar do plano real
        let other_deductions = 0.0;

        let total_deductions = inss + irrf + health_plan + other_deductions;
        let net_salary = gross_salary - total_deductions;

        // Encargos patronais
        employer_inss += base_salary * 0.20; // 20% INSS patronal
        employer_fgts += base_salary * 0.08; // 8% FGTS

        total_gross += gross_salary;
        total_deductions_sum += total_deductions;
        total_net += net_salary;

        payroll_calculations.push(PayrollCalculation {
            employee_id: employee.id,
            name: employee.full_name,
            gross_salary,
            overtime,
            benefits,
            deductions: PayrollDeductions {
                inss,
                irrf,
                health_plan,
                other: other_deductions,
            },
            net_salary,
        });
    }

    Ok(Json(PayrollCalculationResponse {
        reference_month: payload.reference_month,
        total_gross,
        total_deductions: total_deductions_sum,
        total_net,
        employer_charges: EmployerCharges {
            inss: employer_inss,
            fgts: employer_fgts,
            total: employer_inss + employer_fgts,
        },
        employees: payroll_calculations,
    }))
}

async fn get_payroll_by_month(
    claims: Claims,
    State(pool): State<Arc<DbPool>>,
    Path(month): Path<String>,
) -> Result<Json<Vec<serde_json::Value>>> {
    let tenant_id = Uuid::parse_str(&claims.tenant_id)?;

    // TODO: Buscar folhas de pagamento salvas do mês
    Ok(Json(vec![]))
}

async fn get_payroll(
    claims: Claims,
    State(pool): State<Arc<DbPool>>,
    Path(id): Path<Uuid>,
) -> Result<Json<serde_json::Value>> {
    let tenant_id = Uuid::parse_str(&claims.tenant_id)?;

    // TODO: Buscar folha específica
    Err(AppError::not_found("Payroll"))
}

// ============================================================================
// ATTENDANCE (Ponto Eletrônico)
// ============================================================================

#[derive(Debug, serde::Deserialize)]
struct RegisterAttendanceRequest {
    employee_id: Uuid,
    date: NaiveDate,
    check_in_1: Option<chrono::NaiveTime>,
    check_out_1: Option<chrono::NaiveTime>,
    check_in_2: Option<chrono::NaiveTime>,
    check_out_2: Option<chrono::NaiveTime>,
    status: Option<String>, // present, absent, on_leave, holiday
    notes: Option<String>,
}

async fn register_attendance(
    claims: Claims,
    State(pool): State<Arc<DbPool>>,
    Json(payload): Json<RegisterAttendanceRequest>,
) -> Result<(StatusCode, Json<serde_json::Value>)> {
    let tenant_id = Uuid::parse_str(&claims.tenant_id)?;

    // Calcular total de horas
    let total_hours = calculate_work_hours(
        payload.check_in_1,
        payload.check_out_1,
        payload.check_in_2,
        payload.check_out_2,
    );

    // TODO: Salvar no banco de dados
    // INSERT INTO hr_attendance ...

    tracing::info!(
        "Attendance registered: employee={}, date={}, hours={}",
        payload.employee_id,
        payload.date,
        total_hours
    );

    Ok((StatusCode::CREATED, Json(serde_json::json!({
        "employee_id": payload.employee_id,
        "date": payload.date,
        "total_hours": total_hours,
        "status": payload.status.unwrap_or("present".to_string())
    }))))
}

async fn list_attendance(
    claims: Claims,
    State(pool): State<Arc<DbPool>>,
    Query(query): Query<serde_json::Value>,
) -> Result<Json<Vec<serde_json::Value>>> {
    let tenant_id = Uuid::parse_str(&claims.tenant_id)?;

    // TODO: Listar pontos com filtros
    Ok(Json(vec![]))
}

async fn get_attendance(
    claims: Claims,
    State(pool): State<Arc<DbPool>>,
    Path((employee_id, date)): Path<(Uuid, NaiveDate)>,
) -> Result<Json<serde_json::Value>> {
    let tenant_id = Uuid::parse_str(&claims.tenant_id)?;

    // TODO: Buscar ponto específico
    Ok(Json(serde_json::json!({
        "employee_id": employee_id,
        "date": date
    })))
}

async fn attendance_report(
    claims: Claims,
    State(pool): State<Arc<DbPool>>,
    Query(query): Query<serde_json::Value>,
) -> Result<Json<serde_json::Value>> {
    let tenant_id = Uuid::parse_str(&claims.tenant_id)?;

    // TODO: Gerar relatório de ponto (mensal, por funcionário, etc)
    Ok(Json(serde_json::json!({
        "report_type": "monthly",
        "total_employees": 0,
        "total_hours": 0
    })))
}

// ============================================================================
// PERFORMANCE REVIEWS
// ============================================================================

async fn create_performance_review(
    claims: Claims,
    State(pool): State<Arc<DbPool>>,
    Json(payload): Json<serde_json::Value>,
) -> Result<(StatusCode, Json<serde_json::Value>)> {
    let tenant_id = Uuid::parse_str(&claims.tenant_id)?;

    // TODO: Implementar criação de avaliação
    Err(AppError::BadRequest("Not implemented yet".to_string()))
}

async fn list_performance_reviews(
    claims: Claims,
    State(pool): State<Arc<DbPool>>,
) -> Result<Json<Vec<serde_json::Value>>> {
    Ok(Json(vec![]))
}

async fn get_performance_review(
    claims: Claims,
    State(pool): State<Arc<DbPool>>,
    Path(id): Path<Uuid>,
) -> Result<Json<serde_json::Value>> {
    Err(AppError::not_found("Performance Review"))
}

async fn update_performance_review(
    claims: Claims,
    State(pool): State<Arc<DbPool>>,
    Path(id): Path<Uuid>,
    Json(payload): Json<serde_json::Value>,
) -> Result<Json<serde_json::Value>> {
    Err(AppError::not_found("Performance Review"))
}

// ============================================================================
// ANALYTICS
// ============================================================================

async fn turnover_analytics(
    claims: Claims,
    State(pool): State<Arc<DbPool>>,
) -> Result<Json<serde_json::Value>> {
    let tenant_id = Uuid::parse_str(&claims.tenant_id)?;

    // TODO: Calcular turnover rate
    Ok(Json(serde_json::json!({
        "period": "2024",
        "turnover_rate": 12.5,
        "voluntary": 8.0,
        "involuntary": 4.5
    })))
}

async fn headcount_analytics(
    claims: Claims,
    State(pool): State<Arc<DbPool>>,
) -> Result<Json<serde_json::Value>> {
    let tenant_id = Uuid::parse_str(&claims.tenant_id)?;

    let total = sqlx::query_scalar::<_, i64>(
        r#"
        SELECT COUNT(*)
        FROM hr_employees
        WHERE tenant_id = $1 AND status = $2::employee_status
        "#
    )
    .bind(tenant_id)
    .bind(EmployeeStatus::Active as EmployeeStatus)
    .fetch_one(pool.as_ref())
    .await?;

    // Breakdown por departamento
    let by_department = sqlx::query!(
        r#"
        SELECT department, COUNT(*) as "count!"
        FROM hr_employees
        WHERE tenant_id = $1 AND status = $2::employee_status
        GROUP BY department
        "#,
        tenant_id,
        EmployeeStatus::Active as EmployeeStatus
    )
    .fetch_all(pool.as_ref())
    .await?;

    let mut departments = serde_json::Map::new();
    for row in by_department {
        if let Some(dept) = row.department {
            departments.insert(dept, serde_json::json!(row.count));
        }
    }

    Ok(Json(serde_json::json!({
        "total": total,
        "by_department": departments,
        "trend": "growing"
    })))
}

// ============================================================================
// HELPERS
// ============================================================================

/// Validar CPF brasileiro
fn validate_cpf(cpf: &str) -> Result<()> {
    // Remover pontuação
    let cpf_digits: String = cpf.chars().filter(|c| c.is_numeric()).collect();
    
    if cpf_digits.len() != 11 {
        return Err(AppError::validation_error("CPF deve ter 11 dígitos"));
    }

    // TODO: Implementar validação completa de CPF
    // Por enquanto, apenas verificar comprimento
    
    Ok(())
}

/// Calcular INSS
fn calculate_inss(base_salary: f64) -> f64 {
    // Tabela 2024 simplificada
    if base_salary <= 1412.00 {
        base_salary * 0.075
    } else if base_salary <= 2666.68 {
        base_salary * 0.09
    } else if base_salary <= 4000.03 {
        base_salary * 0.12
    } else if base_salary <= 7786.02 {
        base_salary * 0.14
    } else {
        1089.04 // Teto
    }
}

/// Calcular IRRF
fn calculate_irrf(taxable_income: f64) -> f64 {
    // Tabela 2024 simplificada
    if taxable_income <= 2259.20 {
        0.0
    } else if taxable_income <= 2826.65 {
        (taxable_income * 0.075) - 169.44
    } else if taxable_income <= 3751.05 {
        (taxable_income * 0.15) - 381.44
    } else if taxable_income <= 4664.68 {
        (taxable_income * 0.225) - 662.77
    } else {
        (taxable_income * 0.275) - 896.00
    }
}

/// Calcular horas extras do mês
async fn calculate_overtime(pool: &DbPool, employee_id: Uuid, month: &str) -> Result<f64> {
    // TODO: Buscar do ponto eletrônico e calcular
    Ok(0.0)
}

/// Extrair valor total dos benefícios
fn extract_benefits_value(benefits: &serde_json::Value) -> f64 {
    if let Some(obj) = benefits.as_object() {
        obj.values()
            .filter_map(|v| v.as_f64())
            .sum()
    } else {
        0.0
    }
}

/// Calcular horas trabalhadas
fn calculate_work_hours(
    check_in_1: Option<chrono::NaiveTime>,
    check_out_1: Option<chrono::NaiveTime>,
    check_in_2: Option<chrono::NaiveTime>,
    check_out_2: Option<chrono::NaiveTime>,
) -> f64 {
    let mut total_hours = 0.0;

    if let (Some(in1), Some(out1)) = (check_in_1, check_out_1) {
        let duration = out1.signed_duration_since(in1);
        total_hours += duration.num_minutes() as f64 / 60.0;
    }

    if let (Some(in2), Some(out2)) = (check_in_2, check_out_2) {
        let duration = out2.signed_duration_since(in2);
        total_hours += duration.num_minutes() as f64 / 60.0;
    }

    total_hours
}
