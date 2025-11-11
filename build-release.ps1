# Script de Build para Controle Roncatin
# Uso: .\build-release.ps1 -Platform [Android|Windows|iOS|All]

param(
    [Parameter(Mandatory=$true)]
    [ValidateSet("Android", "Windows", "iOS", "MacCatalyst", "All")]
    [string]$Platform
)

$ErrorActionPreference = "Stop"
$ProjectName = "Controle-Roncatin.csproj"

Write-Host "========================================" -ForegroundColor Cyan
Write-Host " Build Release - Controle Roncatin" -ForegroundColor Cyan
Write-Host " Plataforma: $Platform" -ForegroundColor Cyan
Write-Host "========================================" -ForegroundColor Cyan
Write-Host ""

function Build-Android {
 Write-Host "?? Compilando Android (AAB para Google Play)..." -ForegroundColor Green
    dotnet publish $ProjectName `
        -f net9.0-android `
        -c Release `
        -p:AndroidPackageFormat=aab `
        -o .\bin\Release\Android
    
    Write-Host "? Android AAB gerado em: .\bin\Release\Android\" -ForegroundColor Green
    Write-Host ""
}

function Build-Windows {
    Write-Host "?? Compilando Windows..." -ForegroundColor Green
    dotnet publish $ProjectName `
        -f net9.0-windows10.0.19041.0 `
-c Release `
        -p:RuntimeIdentifierOverride=win10-x64 `
        -o .\bin\Release\Windows
    
    Write-Host "? Windows executável gerado em: .\bin\Release\Windows\" -ForegroundColor Green
    Write-Host ""
}

function Build-iOS {
    if ($IsMacOS) {
        Write-Host "?? Compilando iOS..." -ForegroundColor Green
        dotnet build $ProjectName `
  -f net9.0-ios `
        -c Release
        
        Write-Host "? iOS compilado (necessário Xcode para publicar)" -ForegroundColor Green
    } else {
   Write-Host "??  Build de iOS requer macOS com Xcode" -ForegroundColor Yellow
    }
    Write-Host ""
}

function Build-MacCatalyst {
    if ($IsMacOS) {
        Write-Host "???  Compilando macOS (Catalyst)..." -ForegroundColor Green
        dotnet build $ProjectName `
        -f net9.0-maccatalyst `
            -c Release
 
        Write-Host "? macOS compilado" -ForegroundColor Green
    } else {
    Write-Host "??  Build de macOS requer macOS com Xcode" -ForegroundColor Yellow
  }
    Write-Host ""
}

# Executar builds
switch ($Platform) {
    "Android" { Build-Android }
    "Windows" { Build-Windows }
    "iOS" { Build-iOS }
    "MacCatalyst" { Build-MacCatalyst }
    "All" {
        Build-Android
        Build-Windows
        Build-iOS
 Build-MacCatalyst
    }
}

Write-Host "========================================" -ForegroundColor Cyan
Write-Host " ? Build Concluído!" -ForegroundColor Cyan
Write-Host "========================================" -ForegroundColor Cyan
