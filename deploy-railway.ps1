#!/usr/bin/env pwsh
# Deploy to Railway

param(
    [string]$Service = "backend",
    [switch]$Logs,
    [switch]$Status
)

Write-Host "🚂 Railway Deployment Tool" -ForegroundColor Cyan
Write-Host "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━" -ForegroundColor Gray
Write-Host ""

# Check if railway CLI is installed
if (!(Get-Command railway -ErrorAction SilentlyContinue)) {
    Write-Host "❌ Railway CLI não encontrado" -ForegroundColor Red
    Write-Host ""
    Write-Host "Instalação:" -ForegroundColor Yellow
    Write-Host "  npm install -g @railway/cli" -ForegroundColor White
    Write-Host ""
    Write-Host "Ou baixe em: https://railway.app/cli" -ForegroundColor Cyan
    exit 1
}

if ($Logs) {
    Write-Host "📋 Buscando logs do serviço '$Service'..." -ForegroundColor Yellow
    railway logs --service $Service
    exit 0
}

if ($Status) {
    Write-Host "📊 Status dos serviços:" -ForegroundColor Yellow
    railway status
    exit 0
}

# Pre-deployment checks
Write-Host "1️⃣  Verificando workspace..." -ForegroundColor Yellow

if (!(Test-Path "arcsat-backend/Cargo.toml")) {
    Write-Host "   ❌ arcsat-backend não encontrado" -ForegroundColor Red
    exit 1
}

if (!(Test-Path "Dockerfile.railway")) {
    Write-Host "   ❌ Dockerfile.railway não encontrado" -ForegroundColor Red
    exit 1
}

if (!(Test-Path "railway.json")) {
    Write-Host "   ❌ railway.json não encontrado" -ForegroundColor Red
    exit 1
}

Write-Host "   ✅ Workspace válido" -ForegroundColor Green

# Check if linked to Railway project
Write-Host ""
Write-Host "2️⃣  Verificando projeto Railway..." -ForegroundColor Yellow

try {
    $railwayStatus = railway status 2>&1
    if ($LASTEXITCODE -ne 0) {
        Write-Host "   ❌ Projeto não conectado ao Railway" -ForegroundColor Red
        Write-Host ""
        Write-Host "Execute:" -ForegroundColor Yellow
        Write-Host "  railway login" -ForegroundColor White
        Write-Host "  railway link" -ForegroundColor White
        exit 1
    }
    Write-Host "   ✅ Projeto conectado" -ForegroundColor Green
} catch {
    Write-Host "   ❌ Erro ao verificar Railway" -ForegroundColor Red
    exit 1
}

# Confirm deployment
Write-Host ""
Write-Host "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━" -ForegroundColor Gray
Write-Host "⚠️  DEPLOY PARA PRODUÇÃO" -ForegroundColor Yellow
Write-Host "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━" -ForegroundColor Gray
Write-Host ""
Write-Host "Serviço: $Service" -ForegroundColor Cyan
Write-Host "Branch: $(git branch --show-current)" -ForegroundColor Cyan
Write-Host ""
$confirm = Read-Host "Confirmar deploy? (s/n)"

if ($confirm -ne "s" -and $confirm -ne "S") {
    Write-Host "❌ Deploy cancelado" -ForegroundColor Red
    exit 0
}

# Deploy
Write-Host ""
Write-Host "3️⃣  Iniciando deploy..." -ForegroundColor Yellow
Write-Host "   (Isso pode demorar 10-15 minutos na primeira vez)" -ForegroundColor Gray
Write-Host ""

railway up

if ($LASTEXITCODE -eq 0) {
    Write-Host ""
    Write-Host "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━" -ForegroundColor Gray
    Write-Host "✅ Deploy concluído!" -ForegroundColor Green
    Write-Host ""
    Write-Host "Verificar logs:" -ForegroundColor Cyan
    Write-Host "  railway logs --service $Service" -ForegroundColor White
    Write-Host ""
    Write-Host "Status:" -ForegroundColor Cyan
    Write-Host "  railway status" -ForegroundColor White
    Write-Host ""
    Write-Host "URLs dos serviços:" -ForegroundColor Cyan
    Write-Host "  Backend: https://backend-production-5b7d.up.railway.app" -ForegroundColor White
    Write-Host "  Core: https://core-production-028a.up.railway.app" -ForegroundColor White
    Write-Host "  Django App: https://djangoapp-production-62bd.up.railway.app" -ForegroundColor White
    Write-Host ""
    Write-Host "Testar:" -ForegroundColor Cyan
    Write-Host "  .\check-railway-services.ps1" -ForegroundColor White
    Write-Host "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━" -ForegroundColor Gray
} else {
    Write-Host ""
    Write-Host "❌ Deploy falhou" -ForegroundColor Red
    Write-Host ""
    Write-Host "Verificar logs:" -ForegroundColor Yellow
    Write-Host "  railway logs --service $Service" -ForegroundColor White
    Write-Host ""
    Write-Host "Troubleshooting: Ver RAILWAY_SERVICES.md" -ForegroundColor Cyan
    exit 1
}
