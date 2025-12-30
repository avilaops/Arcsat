#!/usr/bin/env pwsh
# Git Push + Railway Auto-Deploy

Write-Host "🚀 Arcsat Deploy via GitHub" -ForegroundColor Cyan
Write-Host "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━" -ForegroundColor Gray
Write-Host ""

# Check if git repo
if (!(Test-Path ".git")) {
    Write-Host "❌ Não é um repositório Git" -ForegroundColor Red
    Write-Host ""
    Write-Host "Execute:" -ForegroundColor Yellow
    Write-Host "  git init" -ForegroundColor White
    Write-Host "  git remote add origin https://github.com/avilaops/Arcsat.git" -ForegroundColor White
    exit 1
}

# Check for uncommitted changes
$status = git status --porcelain
if ($status) {
    Write-Host "📝 Mudanças detectadas:" -ForegroundColor Yellow
    git status --short
    Write-Host ""

    $commit = Read-Host "Mensagem do commit (Enter para 'update backend')"
    if ([string]::IsNullOrWhiteSpace($commit)) {
        $commit = "update backend with market intelligence"
    }

    Write-Host ""
    Write-Host "1️⃣  Adicionando arquivos..." -ForegroundColor Yellow
    git add .

    Write-Host "2️⃣  Criando commit..." -ForegroundColor Yellow
    git commit -m $commit

    if ($LASTEXITCODE -ne 0) {
        Write-Host "❌ Falha no commit" -ForegroundColor Red
        exit 1
    }

    Write-Host "   ✅ Commit criado" -ForegroundColor Green
} else {
    Write-Host "ℹ️  Nenhuma mudança para commit" -ForegroundColor Cyan
}

# Get current branch
$branch = git branch --show-current
Write-Host ""
Write-Host "3️⃣  Fazendo push para GitHub..." -ForegroundColor Yellow
Write-Host "   Branch: $branch" -ForegroundColor Cyan

git push origin $branch

if ($LASTEXITCODE -eq 0) {
    Write-Host "   ✅ Push concluído" -ForegroundColor Green
    Write-Host ""
    Write-Host "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━" -ForegroundColor Gray
    Write-Host "✅ Código enviado para GitHub!" -ForegroundColor Green
    Write-Host ""
    Write-Host "🚂 Railway Deploy:" -ForegroundColor Cyan
    Write-Host ""
    Write-Host "Se Railway está conectado ao GitHub:" -ForegroundColor Yellow
    Write-Host "  ✅ Deploy automático iniciado!" -ForegroundColor Green
    Write-Host "  ⏱️  Aguarde 10-15 minutos (primeira compilação)" -ForegroundColor Gray
    Write-Host ""
    Write-Host "Se Railway NÃO está conectado:" -ForegroundColor Yellow
    Write-Host "  1. Acesse: https://railway.app/dashboard" -ForegroundColor White
    Write-Host "  2. New Project → Deploy from GitHub" -ForegroundColor White
    Write-Host "  3. Selecione: avilaops/Arcsat" -ForegroundColor White
    Write-Host "  4. Configure env vars (ver DEPLOY_GUIDE.md)" -ForegroundColor White
    Write-Host ""
    Write-Host "Verificar após deploy:" -ForegroundColor Cyan
    Write-Host "  .\check-railway-services.ps1" -ForegroundColor White
    Write-Host ""
    Write-Host "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━" -ForegroundColor Gray
} else {
    Write-Host ""
    Write-Host "❌ Falha no push" -ForegroundColor Red
    Write-Host ""
    Write-Host "Possíveis causas:" -ForegroundColor Yellow
    Write-Host "  - Sem permissão no repositório" -ForegroundColor White
    Write-Host "  - Branch divergente (fazer pull primeiro)" -ForegroundColor White
    Write-Host "  - Remote não configurado" -ForegroundColor White
    Write-Host ""
    Write-Host "Tentar:" -ForegroundColor Cyan
    Write-Host "  git pull origin $branch --rebase" -ForegroundColor White
    Write-Host "  git push origin $branch" -ForegroundColor White
    exit 1
}
