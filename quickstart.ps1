#!/usr/bin/env pwsh
# Quick Start - Arcsat Market Intelligence

Write-Host "🚀 Arcsat Market Intelligence - Quick Start" -ForegroundColor Cyan
Write-Host "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━" -ForegroundColor Gray
Write-Host ""

# Check prerequisites
Write-Host "📋 Verificando pré-requisitos..." -ForegroundColor Yellow

$prerequisites = @{
    "Rust" = "cargo"
    "Docker" = "docker"
    "Chrome" = "chrome"
}

$missingPrereqs = @()
foreach ($prereq in $prerequisites.GetEnumerator()) {
    if (!(Get-Command $prereq.Value -ErrorAction SilentlyContinue)) {
        $missingPrereqs += $prereq.Key
        Write-Host "   ❌ $($prereq.Key) não encontrado" -ForegroundColor Red
    } else {
        Write-Host "   ✅ $($prereq.Key) instalado" -ForegroundColor Green
    }
}

if ($missingPrereqs.Count -gt 0) {
    Write-Host ""
    Write-Host "⚠️  Faltam dependências:" -ForegroundColor Yellow
    foreach ($missing in $missingPrereqs) {
        Write-Host "   - $missing" -ForegroundColor White
    }
    Write-Host ""
    Write-Host "Instale as dependências e tente novamente." -ForegroundColor Yellow
    exit 1
}

Write-Host ""
Write-Host "1️⃣  Configurando ambiente..." -ForegroundColor Yellow

# Copy .env if not exists
if (!(Test-Path "arcsat-backend\.env")) {
    Copy-Item "arcsat-backend\.env.example" "arcsat-backend\.env"
    Write-Host "   ✅ .env criado" -ForegroundColor Green
} else {
    Write-Host "   ℹ️  .env já existe" -ForegroundColor Cyan
}

Write-Host ""
Write-Host "2️⃣  Iniciando dependências (Docker)..." -ForegroundColor Yellow

# Check if Docker is running
try {
    docker ps > $null 2>&1
    if ($LASTEXITCODE -ne 0) {
        Write-Host "   ❌ Docker não está rodando. Inicie o Docker Desktop." -ForegroundColor Red
        exit 1
    }
} catch {
    Write-Host "   ❌ Erro ao verificar Docker" -ForegroundColor Red
    exit 1
}

# Start PostgreSQL
Write-Host "   Starting PostgreSQL..." -ForegroundColor Cyan
docker run -d --name arcsat-postgres `
    -p 5432:5432 `
    -e POSTGRES_PASSWORD=arcsat `
    -e POSTGRES_USER=arcsat `
    -e POSTGRES_DB=arcsat `
    postgres:15-alpine > $null 2>&1

# Start Redis
Write-Host "   Starting Redis..." -ForegroundColor Cyan
docker run -d --name arcsat-redis `
    -p 6379:6379 `
    redis:7-alpine > $null 2>&1

Start-Sleep -Seconds 3
Write-Host "   ✅ Postgres e Redis rodando" -ForegroundColor Green

Write-Host ""
Write-Host "3️⃣  Compilando backend..." -ForegroundColor Yellow
Write-Host "   (Primeira compilação pode demorar ~5 minutos)" -ForegroundColor Gray

cd arcsat-backend
cargo build --release --bin arcsat-server --bin arcsat-worker

if ($LASTEXITCODE -ne 0) {
    Write-Host "   ❌ Falha na compilação" -ForegroundColor Red
    exit 1
}

Write-Host "   ✅ Compilação concluída" -ForegroundColor Green

Write-Host ""
Write-Host "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━" -ForegroundColor Gray
Write-Host "✅ Setup completo!" -ForegroundColor Green
Write-Host ""
Write-Host "Para iniciar o sistema:" -ForegroundColor Cyan
Write-Host ""
Write-Host "Terminal 1 - API Server:" -ForegroundColor Yellow
Write-Host "  cd arcsat-backend" -ForegroundColor White
Write-Host "  cargo run --release --bin arcsat-server" -ForegroundColor White
Write-Host ""
Write-Host "Terminal 2 - Worker (opcional):" -ForegroundColor Yellow
Write-Host "  cd arcsat-backend" -ForegroundColor White
Write-Host "  cargo run --release --bin arcsat-worker" -ForegroundColor White
Write-Host ""
Write-Host "Testar:" -ForegroundColor Yellow
Write-Host "  .\test-market-intelligence.ps1" -ForegroundColor White
Write-Host ""
Write-Host "Dashboard:" -ForegroundColor Yellow
Write-Host "  Abra: dashboard-market-intelligence.html" -ForegroundColor White
Write-Host ""
Write-Host "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━" -ForegroundColor Gray
