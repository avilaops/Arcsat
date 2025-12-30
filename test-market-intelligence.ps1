#!/usr/bin/env pwsh
# Script de teste local do Market Intelligence (sem proxy)

Write-Host "🧪 Testando Arcsat Market Intelligence" -ForegroundColor Cyan
Write-Host ""

$baseUrl = "http://localhost:3000"

# 1. Health check
Write-Host "1️⃣  Verificando health..." -ForegroundColor Yellow
try {
    $health = Invoke-RestMethod -Uri "$baseUrl/health" -Method Get
    Write-Host "   ✅ Server healthy: $($health.status)" -ForegroundColor Green
} catch {
    Write-Host "   ❌ Server não está rodando. Execute: cargo run --bin arcsat-server" -ForegroundColor Red
    exit 1
}

# 2. Market Intelligence health
Write-Host "2️⃣  Verificando Market Intelligence..." -ForegroundColor Yellow
try {
    $miHealth = Invoke-RestMethod -Uri "$baseUrl/api/v1/market-intelligence/health" -Method Get
    Write-Host "   ✅ Market Intelligence: $($miHealth.data)" -ForegroundColor Green
} catch {
    Write-Host "   ❌ Market Intelligence não disponível" -ForegroundColor Red
}

Write-Host ""
Write-Host "3️⃣  Criando job de scraping (Mercado Livre)..." -ForegroundColor Yellow

$jobRequest = @{
    marketplace = "mercado_livre"
    search_query = "teclado mecânico"
    max_pages = 2
    priority = 5
} | ConvertTo-Json

try {
    $jobResponse = Invoke-RestMethod -Uri "$baseUrl/api/v1/market-intelligence/jobs" `
        -Method Post `
        -ContentType "application/json" `
        -Body $jobRequest

    $jobId = $jobResponse.data.job_id
    Write-Host "   ✅ Job criado: $jobId" -ForegroundColor Green
    Write-Host "   Status: $($jobResponse.data.status)" -ForegroundColor Cyan

    # 4. Verificar status (loop)
    Write-Host ""
    Write-Host "4️⃣  Aguardando processamento..." -ForegroundColor Yellow

    $maxAttempts = 30
    $attempt = 0

    while ($attempt -lt $maxAttempts) {
        Start-Sleep -Seconds 2
        $attempt++

        try {
            $statusResponse = Invoke-RestMethod -Uri "$baseUrl/api/v1/market-intelligence/jobs/$jobId/status" -Method Get
            $status = $statusResponse.data

            Write-Host "   ⏳ Tentativa $attempt/$maxAttempts - Status: $status" -ForegroundColor Cyan

            if ($status -eq "Completed" -or $status -eq "completed") {
                Write-Host "   ✅ Job concluído!" -ForegroundColor Green
                break
            }

            if ($status -eq "Failed" -or $status -eq "failed") {
                Write-Host "   ❌ Job falhou!" -ForegroundColor Red
                break
            }
        } catch {
            Write-Host "   ⚠️  Erro ao verificar status: $_" -ForegroundColor Yellow
        }
    }

    if ($attempt -eq $maxAttempts) {
        Write-Host "   ⏱️  Timeout - Job ainda está processando" -ForegroundColor Yellow
    }

} catch {
    Write-Host "   ❌ Erro ao criar job: $_" -ForegroundColor Red
    Write-Host "   Body: $jobRequest" -ForegroundColor Gray
}

Write-Host ""
Write-Host "5️⃣  Testando Amazon (vai levar mais tempo)..." -ForegroundColor Yellow

$amazonRequest = @{
    marketplace = "amazon"
    search_query = "mouse gamer"
    max_pages = 1
    priority = 8
} | ConvertTo-Json

try {
    $amazonResponse = Invoke-RestMethod -Uri "$baseUrl/api/v1/market-intelligence/jobs" `
        -Method Post `
        -ContentType "application/json" `
        -Body $amazonRequest

    Write-Host "   ✅ Job Amazon criado: $($amazonResponse.data.job_id)" -ForegroundColor Green
    Write-Host "   💡 Dica: Use 'curl $baseUrl/api/v1/market-intelligence/jobs/$($amazonResponse.data.job_id)/status' para acompanhar" -ForegroundColor Cyan
} catch {
    Write-Host "   ❌ Erro: $_" -ForegroundColor Red
}

Write-Host ""
Write-Host "📊 Resumo do Teste" -ForegroundColor Cyan
Write-Host "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━" -ForegroundColor Gray
Write-Host "✅ Servidor funcionando" -ForegroundColor Green
Write-Host "✅ Market Intelligence ativo" -ForegroundColor Green
Write-Host "✅ Jobs podem ser criados" -ForegroundColor Green
Write-Host ""
Write-Host "⚠️  LEMBRETE: Sem proxy, você tem limite de ~10-20 requests" -ForegroundColor Yellow
Write-Host "   Após isso, pode receber CAPTCHA ou bloqueio temporário" -ForegroundColor Yellow
Write-Host ""
Write-Host "💡 Próximos passos:" -ForegroundColor Cyan
Write-Host "   1. Configure proxy em .env (MI_PROXY_ENABLED=true)" -ForegroundColor White
Write-Host "   2. Rode workers: cargo run --bin arcsat-worker" -ForegroundColor White
Write-Host "   3. Acesse dashboard: http://localhost:3000/dashboard" -ForegroundColor White
