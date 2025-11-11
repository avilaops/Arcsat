# ?? Guia de Deploy - Controle Roncatin

## Identificação do App
- **Nome:** Controle Roncatin
- **ID:** `inc.avila.roncatincontrole`
- **Versão:** 1.0 (build 1)
- **Domínio:** avila.inc

---

## ?? Deploy Android (Google Play)

### Pré-requisitos
1. Conta Google Play Developer ($25 taxa única)
2. Keystore para assinatura
3. Privacy Policy publicada online

### 1. Gerar Keystore (primeira vez)
```powershell
keytool -genkeypair -v `
  -keystore controle-roncatin.keystore `
  -alias roncatin `
  -keyalg RSA `
  -keysize 2048 `
  -validity 10000 `
  -storepass SUA_SENHA_AQUI `
  -keypass SUA_SENHA_AQUI `
  -dname "CN=Avila Inc, OU=Development, O=Avila, L=Cidade, ST=Estado, C=BR"
```

?? **IMPORTANTE:** Guarde o arquivo `.keystore` e as senhas em local SEGURO! Perder isso significa não poder mais atualizar o app!

### 2. Configurar assinatura no projeto
Edite `Controle-Roncatin.csproj` e adicione:

```xml
<PropertyGroup Condition="'$(Configuration)' == 'Release' and '$(TargetFramework)' == 'net9.0-android'">
  <AndroidKeyStore>true</AndroidKeyStore>
  <AndroidSigningKeyStore>controle-roncatin.keystore</AndroidSigningKeyStore>
  <AndroidSigningKeyAlias>roncatin</AndroidSigningKeyAlias>
  <AndroidSigningKeyPass>SUA_SENHA</AndroidSigningKeyPass>
  <AndroidSigningStorePass>SUA_SENHA</AndroidSigningStorePass>
</PropertyGroup>
```

### 3. Compilar AAB (Android App Bundle)
```powershell
.\build-release.ps1 -Platform Android
```

Ou manualmente:
```powershell
dotnet publish Controle-Roncatin.csproj `
  -f net9.0-android `
  -c Release `
  -p:AndroidPackageFormat=aab
```

### 4. Publicar no Google Play Console
1. Acesse [Google Play Console](https://play.google.com/console)
2. Criar novo aplicativo
3. Preencher ficha da loja:
   - Nome: Controle Roncatin
   - Descrição curta e completa
   - Ícone (512x512 PNG)
   - Screenshots (mínimo 2 por dispositivo)
   - Categoria: Produtividade / Empresarial
4. Upload do AAB em "Versão de produção"
5. Configurar público-alvo e conteúdo
6. Enviar para análise

---

## ?? Deploy Windows

### Opção 1: Distribuição Direta (Sem Microsoft Store)
```powershell
.\build-release.ps1 -Platform Windows
```

Distribua a pasta `bin\Release\Windows\` compactada.

### Opção 2: Microsoft Store (MSIX)

#### 1. Alterar projeto para MSIX
Edite `Controle-Roncatin.csproj`:
```xml
<WindowsPackageType>MSIX</WindowsPackageType>
```

#### 2. Criar certificado de teste (desenvolvimento)
```powershell
New-SelfSignedCertificate `
  -Type Custom `
  -Subject "CN=Avila Inc" `
  -KeyUsage DigitalSignature `
  -FriendlyName "Controle Roncatin Dev Cert" `
  -CertStoreLocation "Cert:\CurrentUser\My"
```

#### 3. Compilar MSIX
```powershell
dotnet publish Controle-Roncatin.csproj `
  -f net9.0-windows10.0.19041.0 `
  -c Release `
  -p:RuntimeIdentifierOverride=win10-x64 `
  -p:GenerateAppxPackageOnBuild=true
```

#### 4. Publicar na Microsoft Store
1. Acesse [Partner Center](https://partner.microsoft.com/dashboard)
2. Criar nova app (reserva o nome)
3. Upload do `.msix` ou `.msixbundle`
4. Preencher descrição, screenshots, etc.
5. Enviar para certificação

---

## ?? Deploy iOS (App Store)

### Pré-requisitos
- Mac com Xcode instalado
- Apple Developer Account ($99/ano)
- Provisioning Profile

### 1. Configurar no Mac
```bash
dotnet build Controle-Roncatin.csproj \
  -f net9.0-ios \
  -c Release
```

### 2. Abrir no Xcode
```bash
open ./bin/Release/net9.0-ios/ios-arm64/Controle_Roncatin.app
```

### 3. Configurar assinatura no Xcode
1. Signing & Capabilities
2. Selecionar Team (Apple Developer Account)
3. Provisioning Profile automático ou manual

### 4. Arquivar e enviar
1. Product > Archive
2. Distribute App > App Store Connect
3. Upload

### 5. App Store Connect
1. Acesse [App Store Connect](https://appstoreconnect.apple.com)
2. Criar novo app
3. Preencher metadados
4. Enviar para análise

---

## ??? Deploy macOS (App Store ou Notarização)

Similar ao iOS, mas requer:
- Notarização (obrigatória, mesmo fora da App Store)
- Certificado "Developer ID Application"

```bash
dotnet build Controle-Roncatin.csproj \
  -f net9.0-maccatalyst \
  -c Release
```

---

## ?? Checklist Pré-Deploy (Todas as Plataformas)

- [ ] Ícones personalizados criados (não usar os padrões do template)
- [ ] Splash screen personalizado
- [ ] Privacy Policy publicada (URL acessível)
- [ ] Screenshots/capturas de tela preparadas
- [ ] Descrição do app escrita
- [ ] Versão e build number atualizados
- [ ] Testado em modo Release (não apenas Debug)
- [ ] Banco de dados SQLite testado em produção
- [ ] Funcionalidades principais testadas
- [ ] Certificados/keystore guardados em local seguro

---

## ?? Atualizações Futuras

### Incrementar versão
Edite `Controle-Roncatin.csproj`:

```xml
<!-- Para atualizações pequenas (1.0 -> 1.1) -->
<ApplicationDisplayVersion>1.1</ApplicationDisplayVersion>
<ApplicationVersion>2</ApplicationVersion>

<!-- Para atualizações grandes (1.0 -> 2.0) -->
<ApplicationDisplayVersion>2.0</ApplicationDisplayVersion>
<ApplicationVersion>3</ApplicationVersion>
```

**Regras:**
- `ApplicationVersion` (build number): sempre **aumenta** (inteiro)
- `ApplicationDisplayVersion` (versão pública): formato semântico (1.0, 1.1, 2.0)

---

## ?? Suporte

Para problemas de build ou deploy, consulte:
- [.NET MAUI Docs](https://learn.microsoft.com/dotnet/maui/)
- [Android Developer](https://developer.android.com/)
- [Apple Developer](https://developer.apple.com/)
- [Microsoft Store](https://learn.microsoft.com/windows/apps/publish/)

---

## ?? IMPORTANTE - Backup

Faça backup de:
1. **Android:** `controle-roncatin.keystore` + senhas
2. **iOS/macOS:** Certificados e Provisioning Profiles
3. **Windows:** Certificado de assinatura (`.pfx`)

**Perder esses arquivos = impossível atualizar o app nas lojas!**
