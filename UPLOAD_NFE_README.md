# 📄 Upload de XML - NFe (Nota Fiscal Eletrônica)

Sistema completo para upload, processamento e gestão de Notas Fiscais Eletrônicas (NFe) via XML.

## 🚀 Funcionalidades

- ✅ Upload de arquivos XML de NFe
- ✅ Suporte a múltiplos arquivos simultâneos
- ✅ Drag & Drop para facilitar o upload
- ✅ Parse automático do XML da NFe
- ✅ Extração de todos os dados relevantes (emitente, destinatário, valores, impostos, itens)
- ✅ Armazenamento em banco de dados SQLite
- ✅ Interface visual para listagem e consulta
- ✅ Download do XML original
- ✅ Filtros por tipo (entrada/saída), status e pesquisa
- ✅ Dashboard com estatísticas

## 📋 Estrutura Implementada

### Backend (Rust/Axum)

```
core/backend/
├── src/
│   ├── models/
│   │   └── nfe.rs          # Estruturas da NFe e XML parsing
│   ├── routes/
│   │   └── nfe.rs          # Endpoints de upload e consulta
│   └── main.rs             # Configuração das rotas
├── migrations/
│   └── 20241228000000_nfe_tables.sql  # Schema do banco
└── Cargo.toml              # Dependências (quick-xml, serde-xml-rs)
```

### Frontend (HTML/JavaScript)

```
core/frontend/
├── upload-nfe.html         # Interface de upload
└── nfes.html              # Listagem e gestão de NFes
```

## 🔧 Configuração

### 1. Instalar Dependências

As dependências já foram adicionadas ao `Cargo.toml`:
- `quick-xml` - Parse rápido de XML
- `serde-xml-rs` - Serialização/deserialização XML

### 2. Executar Migrations

As migrations serão executadas automaticamente ao iniciar o backend.

### 3. Iniciar o Backend

```bash
cd core/backend
cargo run
```

O servidor estará disponível em `http://localhost:3000`

### 4. Acessar o Frontend

Abra no navegador:
- Upload: `http://localhost:3000/upload-nfe.html`
- Listagem: `http://localhost:3000/nfes.html`

## 📡 API Endpoints

### Upload de NFe
```http
POST /api/v1/nfe/upload
Content-Type: multipart/form-data

Parâmetros:
- xml: arquivo XML da NFe
- tipo: "entrada" ou "saida"
- empresa_id: UUID da empresa (opcional)

Resposta:
{
  "success": true,
  "nfe_id": "uuid",
  "chave_acesso": "44210...",
  "numero": "123456",
  "valor_total": 1500.00
}
```

### Listar NFes
```http
GET /api/v1/nfe?page=1&limit=20&tipo=entrada

Resposta: Array de NFes
```

### Buscar NFe por ID
```http
GET /api/v1/nfe/{id}

Resposta: Objeto NFe completo
```

### Buscar Itens da NFe
```http
GET /api/v1/nfe/{id}/items

Resposta: Array de itens
```

### Download do XML
```http
GET /api/v1/nfe/{id}/xml

Resposta: Conteúdo XML original
```

## 💾 Estrutura do Banco de Dados

### Tabela: `nfes`

Armazena os dados principais da NFe:
- Identificação (número, série, chave de acesso)
- Emitente e destinatário
- Valores (total, produtos, impostos)
- Datas
- XML completo
- Status

### Tabela: `nfe_items`

Armazena os itens da NFe:
- Produto (código, descrição, NCM, CFOP)
- Quantidades e valores
- Impostos por item

## 🎨 Interface do Usuário

### Página de Upload

- **Drag & Drop**: Arraste arquivos XML diretamente
- **Seleção Manual**: Clique para selecionar arquivos
- **Múltiplos Arquivos**: Processe vários XMLs de uma vez
- **Tipo de NFe**: Escolha entre Entrada ou Saída
- **Barra de Progresso**: Acompanhe o processamento
- **Resultados Detalhados**: Visualize sucesso/erro de cada arquivo

### Página de Listagem

- **Dashboard**: Estatísticas de entrada, saída e saldo
- **Filtros**: Por tipo, status e pesquisa livre
- **Tabela Completa**: Todas as NFes com dados principais
- **Ações**: Visualizar detalhes e baixar XML

## 📊 Dados Extraídos do XML

O sistema extrai automaticamente:

### Identificação
- Número e série da nota
- Chave de acesso (44 dígitos)
- Data de emissão
- Natureza da operação
- Finalidade

### Partes Envolvidas
- **Emitente**: CNPJ, razão social, nome fantasia
- **Destinatário**: CPF/CNPJ, nome

### Valores
- Valor total da nota
- Valor dos produtos
- ICMS
- IPI
- PIS
- COFINS
- Frete, seguro, desconto

### Itens
- Código do produto
- Descrição
- NCM (Nomenclatura Comum do Mercosul)
- CFOP (Código Fiscal de Operações)
- Quantidade e valores

## 🔐 Segurança (TODO)

Implementações futuras:
- ✅ Autenticação JWT (já configurado, desabilitado temporariamente)
- ⏳ Validação de permissões por empresa
- ⏳ Auditoria de uploads
- ⏳ Validação de assinatura digital da NFe

## 🧪 Testando

### Exemplo de Teste Manual

1. Obtenha um arquivo XML de NFe válido
2. Acesse `upload-nfe.html`
3. Selecione o tipo (Entrada/Saída)
4. Arraste o XML ou clique para selecionar
5. Aguarde o processamento
6. Verifique os resultados
7. Acesse `nfes.html` para ver a NFe cadastrada

### XML de Teste

Para teste, você pode:
- Usar um XML real de NFe (se tiver acesso)
- Gerar um XML de teste seguindo o layout da NFe 4.0
- Baixar exemplos do site da SEFAZ

## 📝 Próximas Melhorias

- [ ] Validação de assinatura digital
- [ ] Consulta de status na SEFAZ
- [ ] Geração de DANFE (PDF da NFe)
- [ ] Integração com sistema de estoque
- [ ] Conciliação bancária automática
- [ ] Relatórios fiscais (SPED)
- [ ] Exportação para Excel/CSV
- [ ] Webhooks para notificações

## 🐛 Troubleshooting

### Erro ao fazer parse do XML
- Verifique se o XML está bem formado
- Confirme que é um XML de NFe válido (layout 4.0)
- Verifique a encoding (deve ser UTF-8)

### Erro de conexão com API
- Confirme que o backend está rodando
- Verifique se a porta 3000 está disponível
- Confira a URL da API no JavaScript

### Dados não aparecem na listagem
- Execute as migrations: `cargo run` (executa automaticamente)
- Verifique os logs do backend para erros

## 📚 Referências

- [Layout NFe 4.0](http://www.nfe.fazenda.gov.br/portal/principal.aspx)
- [Documentação Axum](https://docs.rs/axum/latest/axum/)
- [Quick-XML](https://docs.rs/quick-xml/latest/quick_xml/)

---

**Desenvolvido com ❤️ usando Rust + Axum + SQLite**
