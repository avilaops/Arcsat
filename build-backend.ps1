#!/usr/bin/env pwsh
# Build script para Arcsat Backend

Write-Host "🏗️  Building Arcsat Backend..." -ForegroundColor Cyan

# Check se Rust está instalado
if (!(Get-Command cargo -ErrorAction SilentlyContinue)) {
    Write-Host "❌ Rust não está instalado. Instale de https://rustup.rs" -ForegroundColor Red
    exit 1
}

# Check se Chrome está instalado (para scraping)
if (!(Get-Command chrome -ErrorAction SilentlyContinue) -and !(Get-Command google-chrome -ErrorAction SilentlyContinue)) {
    Write-Host "⚠️  Chrome não encontrado. O scraping pode não funcionar." -ForegroundColor Yellow
    Write-Host "Instale de https://www.google.com/chrome/" -ForegroundColor Yellow
}

# Entrar no diretório
Set-Location arcsat-backend

# Copiar .env se não existir
if (!(Test-Path .env)) {
    Write-Host "📝 Criando .env de exemplo..." -ForegroundColor Yellow
    Copy-Item .env.example .env
    Write-Host "✅ .env criado. Configure antes de rodar!" -ForegroundColor Green
}

# Build
Write-Host "⚙️  Compilando (pode demorar na primeira vez)..." -ForegroundColor Cyan
cargo build --release --bin arcsat-server

if ($LASTEXITCODE -eq 0) {
    Write-Host "✅ Build concluído com sucesso!" -ForegroundColor Green
    Write-Host ""
    Write-Host "Para rodar:" -ForegroundColor Cyan
    Write-Host "  cd arcsat-backend" -ForegroundColor White
    Write-Host "  .\target\release\arcsat-server.exe" -ForegroundColor White
    Write-Host ""
    Write-Host "Ou:" -ForegroundColor Cyan
    Write-Host "  cargo run --release --bin arcsat-server" -ForegroundColor White
} else {
    Write-Host "❌ Build falhou!" -ForegroundColor Red
    exit 1
}
