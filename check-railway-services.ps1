#!/usr/bin/env pwsh
# Railway Services Health Check

$services = @(
    @{Name="Django App"; URL="https://djangoapp-production-62bd.up.railway.app"},
    @{Name="Core"; URL="https://core-production-028a.up.railway.app"},
    @{Name="Backend"; URL="https://backend-production-5b7d.up.railway.app"}
)

Write-Host "🚂 Railway Services Health Check" -ForegroundColor Cyan
Write-Host "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━" -ForegroundColor Gray
Write-Host ""

foreach ($service in $services) {
    Write-Host "Testing $($service.Name)..." -ForegroundColor Yellow
    Write-Host "  URL: $($service.URL)" -ForegroundColor Gray

    try {
        # Test root endpoint
        $response = Invoke-WebRequest -Uri $service.URL -Method GET -TimeoutSec 10 -ErrorAction Stop
        Write-Host "  ✅ Status: $($response.StatusCode)" -ForegroundColor Green
        Write-Host "  📄 Content-Type: $($response.Headers['Content-Type'])" -ForegroundColor Cyan

        # Try common health endpoints
        $healthEndpoints = @("/health", "/api/health", "/healthz", "/api/v1/health")
        foreach ($endpoint in $healthEndpoints) {
            try {
                $healthUrl = "$($service.URL)$endpoint"
                $healthResponse = Invoke-WebRequest -Uri $healthUrl -Method GET -TimeoutSec 5 -ErrorAction Stop
                Write-Host "  ✅ Health endpoint found: $endpoint" -ForegroundColor Green
                break
            } catch {
                # Continue trying
            }
        }

    } catch {
        $statusCode = $_.Exception.Response.StatusCode.value__
        if ($statusCode -eq 404) {
            Write-Host "  ⚠️  404 Not Found - Service may need routes configured" -ForegroundColor Yellow
        } elseif ($statusCode -eq 502 -or $statusCode -eq 503) {
            Write-Host "  ❌ Service Down (Error $statusCode)" -ForegroundColor Red
        } else {
            Write-Host "  ❌ Error: $($_.Exception.Message)" -ForegroundColor Red
        }
    }

    Write-Host ""
}

Write-Host "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━" -ForegroundColor Gray
Write-Host ""
Write-Host "🔍 Possíveis problemas:" -ForegroundColor Cyan
Write-Host "  1. Dockerfile não está configurado corretamente" -ForegroundColor White
Write-Host "  2. Variáveis de ambiente faltando" -ForegroundColor White
Write-Host "  3. Build falhou no Railway" -ForegroundColor White
Write-Host "  4. Porta não configurada (PORT env var)" -ForegroundColor White
Write-Host ""
Write-Host "💡 Próximos passos:" -ForegroundColor Yellow
Write-Host "  1. Verificar logs no Railway Dashboard" -ForegroundColor White
Write-Host "  2. Checar railway.json e Dockerfile" -ForegroundColor White
Write-Host "  3. Validar variáveis de ambiente" -ForegroundColor White
Write-Host "  4. Re-deploy com: railway up" -ForegroundColor White
