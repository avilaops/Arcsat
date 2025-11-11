using SQLite;
using Controle_Roncatin.Models;

namespace Controle_Roncatin.Services
{
    /// <summary>
    /// Serviço para gerenciamento do banco de dados SQLite
    /// </summary>
    public class DatabaseService
    {
  private SQLiteAsyncConnection? _database;
  private readonly string _dbPath;

        public DatabaseService()
    {
    // Define o caminho do banco de dados no armazenamento local do app
  _dbPath = Path.Combine(FileSystem.AppDataDirectory, "controle_roncatin.db3");
  }

    /// <summary>
        /// Inicializa o banco de dados e cria as tabelas
 /// </summary>
        private async Task InitAsync()
        {
            if (_database != null)
           return;

    _database = new SQLiteAsyncConnection(_dbPath);

         // Criar tabelas
     await _database.CreateTableAsync<Metal>();
        await _database.CreateTableAsync<Compra>();
     await _database.CreateTableAsync<Venda>();
    await _database.CreateTableAsync<MovimentacaoEstoque>();

            // Inserir dados iniciais se necessário
            await SeedDataAsync();
  }

        /// <summary>
     /// Insere dados iniciais (metais comuns no Brasil)
   /// </summary>
      private async Task SeedDataAsync()
        {
       var count = await _database!.Table<Metal>().CountAsync();
            if (count > 0)
     return;

       var metaisIniciais = new List<Metal>
       {
      new Metal { Nome = "Alumínio", Descricao = "Latas, perfis, chapas", UnidadeMedida = "KG", PrecoCompra = 4.50m, PrecoVenda = 6.00m, EstoqueMinimo = 100, Ativo = true },
       new Metal { Nome = "Cobre", Descricao = "Fios, cabos, tubos", UnidadeMedida = "KG", PrecoCompra = 25.00m, PrecoVenda = 32.00m, EstoqueMinimo = 50, Ativo = true },
      new Metal { Nome = "Ferro", Descricao = "Sucata ferrosa", UnidadeMedida = "KG", PrecoCompra = 0.80m, PrecoVenda = 1.20m, EstoqueMinimo = 500, Ativo = true },
new Metal { Nome = "Aço", Descricao = "Estruturas, vigas", UnidadeMedida = "KG", PrecoCompra = 1.00m, PrecoVenda = 1.50m, EstoqueMinimo = 500, Ativo = true },
     new Metal { Nome = "Bronze", Descricao = "Peças, válvulas", UnidadeMedida = "KG", PrecoCompra = 18.00m, PrecoVenda = 24.00m, EstoqueMinimo = 30, Ativo = true },
           new Metal { Nome = "Latão", Descricao = "Torneiras, conexões", UnidadeMedida = "KG", PrecoCompra = 15.00m, PrecoVenda = 20.00m, EstoqueMinimo = 30, Ativo = true },
 new Metal { Nome = "Inox", Descricao = "Aço inoxidável", UnidadeMedida = "KG", PrecoCompra = 3.50m, PrecoVenda = 5.00m, EstoqueMinimo = 100, Ativo = true },
            new Metal { Nome = "Zinco", Descricao = "Chapas, peças", UnidadeMedida = "KG", PrecoCompra = 2.50m, PrecoVenda = 3.50m, EstoqueMinimo = 50, Ativo = true },
         new Metal { Nome = "Chumbo", Descricao = "Baterias, peças", UnidadeMedida = "KG", PrecoCompra = 3.00m, PrecoVenda = 4.50m, EstoqueMinimo = 50, Ativo = true },
    new Metal { Nome = "Níquel", Descricao = "Ligas, baterias", UnidadeMedida = "KG", PrecoCompra = 35.00m, PrecoVenda = 45.00m, EstoqueMinimo = 20, Ativo = true }
   };

            await _database.InsertAllAsync(metaisIniciais);
        }

        // ==================== METAIS ====================
        
        public async Task<List<Metal>> GetMetaisAsync()
        {
         await InitAsync();
       return await _database!.Table<Metal>().Where(m => m.Ativo).OrderBy(m => m.Nome).ToListAsync();
        }

        public async Task<List<Metal>> GetTodosMetaisAsync()
        {
      await InitAsync();
         return await _database!.Table<Metal>().OrderBy(m => m.Nome).ToListAsync();
        }

        public async Task<Metal?> GetMetalAsync(int id)
    {
            await InitAsync();
            return await _database!.Table<Metal>().Where(m => m.Id == id).FirstOrDefaultAsync();
        }

        public async Task<int> SaveMetalAsync(Metal metal)
        {
          await InitAsync();
 if (metal.Id != 0)
         return await _database!.UpdateAsync(metal);
            else
         return await _database!.InsertAsync(metal);
        }

    public async Task<int> DeleteMetalAsync(Metal metal)
        {
      await InitAsync();
    // Soft delete
      metal.Ativo = false;
    return await _database!.UpdateAsync(metal);
        }

        // ==================== COMPRAS ====================
     
        public async Task<List<Compra>> GetComprasAsync(DateTime? dataInicio = null, DateTime? dataFim = null)
        {
   await InitAsync();
         var query = _database!.Table<Compra>();

            if (dataInicio.HasValue)
                query = query.Where(c => c.DataCompra >= dataInicio.Value);
            
 if (dataFim.HasValue)
          query = query.Where(c => c.DataCompra <= dataFim.Value);

            var compras = await query.OrderByDescending(c => c.DataCompra).ToListAsync();
            
         // Carregar metais relacionados
   foreach (var compra in compras)
   {
  compra.Metal = await GetMetalAsync(compra.MetalId);
      }

return compras;
        }

        public async Task<Compra?> GetCompraAsync(int id)
        {
            await InitAsync();
            var compra = await _database!.Table<Compra>().Where(c => c.Id == id).FirstOrDefaultAsync();
        if (compra != null)
            {
compra.Metal = await GetMetalAsync(compra.MetalId);
    }
            return compra;
   }

     public async Task<int> SaveCompraAsync(Compra compra)
        {
 await InitAsync();
            compra.ValorTotal = compra.Quantidade * compra.PrecoUnitario;
            
       int result;
    if (compra.Id != 0)
            {
   result = await _database!.UpdateAsync(compra);
            }
            else
      {
          result = await _database!.InsertAsync(compra);
                
// Registrar movimentação de estoque (entrada)
           await RegistrarMovimentacaoAsync(compra.MetalId, "COMPRA", compra.Quantidade, compra.Id, null);
            }

        return result;
        }

        public async Task<int> DeleteCompraAsync(Compra compra)
        {
    await InitAsync();
// Reverter movimentação de estoque
         await RegistrarMovimentacaoAsync(compra.MetalId, "AJUSTE", -compra.Quantidade, null, null, "Estorno de compra excluída");
   return await _database!.DeleteAsync(compra);
      }

        // ==================== VENDAS ====================
    
        public async Task<List<Venda>> GetVendasAsync(DateTime? dataInicio = null, DateTime? dataFim = null)
        {
  await InitAsync();
  var query = _database!.Table<Venda>();

            if (dataInicio.HasValue)
            query = query.Where(v => v.DataVenda >= dataInicio.Value);
            
   if (dataFim.HasValue)
 query = query.Where(v => v.DataVenda <= dataFim.Value);

    var vendas = await query.OrderByDescending(v => v.DataVenda).ToListAsync();
            
            // Carregar metais relacionados
      foreach (var venda in vendas)
            {
       venda.Metal = await GetMetalAsync(venda.MetalId);
            }

            return vendas;
        }

        public async Task<Venda?> GetVendaAsync(int id)
        {
          await InitAsync();
            var venda = await _database!.Table<Venda>().Where(v => v.Id == id).FirstOrDefaultAsync();
       if (venda != null)
            {
           venda.Metal = await GetMetalAsync(venda.MetalId);
         }
            return venda;
        }

     public async Task<int> SaveVendaAsync(Venda venda)
        {
       await InitAsync();
   venda.ValorTotal = venda.Quantidade * venda.PrecoUnitario;
      
    int result;
            if (venda.Id != 0)
 {
    result = await _database!.UpdateAsync(venda);
       }
  else
            {
  result = await _database!.InsertAsync(venda);
  
          // Registrar movimentação de estoque (saída)
 await RegistrarMovimentacaoAsync(venda.MetalId, "VENDA", -venda.Quantidade, null, venda.Id);
  }

            return result;
        }

        public async Task<int> DeleteVendaAsync(Venda venda)
        {
     await InitAsync();
       // Reverter movimentação de estoque
  await RegistrarMovimentacaoAsync(venda.MetalId, "AJUSTE", venda.Quantidade, null, null, "Estorno de venda excluída");
       return await _database!.DeleteAsync(venda);
        }

 // ==================== MOVIMENTAÇÃO DE ESTOQUE ====================
  
    private async Task RegistrarMovimentacaoAsync(int metalId, string tipo, decimal quantidade, int? compraId = null, int? vendaId = null, string? observacoes = null)
        {
 await InitAsync();
 
            var saldoAnterior = await GetSaldoAtualAsync(metalId);
  var saldoAtual = saldoAnterior + quantidade;

     var movimentacao = new MovimentacaoEstoque
      {
                MetalId = metalId,
     TipoMovimentacao = tipo,
        Quantidade = quantidade,
            SaldoAnterior = saldoAnterior,
      SaldoAtual = saldoAtual,
                CompraId = compraId,
       VendaId = vendaId,
          DataMovimentacao = DateTime.Now,
      Observacoes = observacoes
 };

      await _database!.InsertAsync(movimentacao);
        }

        public async Task<List<MovimentacaoEstoque>> GetMovimentacoesAsync(int? metalId = null, DateTime? dataInicio = null, DateTime? dataFim = null)
        {
            await InitAsync();
 var query = _database!.Table<MovimentacaoEstoque>();

            if (metalId.HasValue)
        query = query.Where(m => m.MetalId == metalId.Value);

        if (dataInicio.HasValue)
    query = query.Where(m => m.DataMovimentacao >= dataInicio.Value);
            
  if (dataFim.HasValue)
             query = query.Where(m => m.DataMovimentacao <= dataFim.Value);

            var movimentacoes = await query.OrderByDescending(m => m.DataMovimentacao).ToListAsync();
   
            // Carregar metais relacionados
     foreach (var mov in movimentacoes)
         {
        mov.Metal = await GetMetalAsync(mov.MetalId);
            }

            return movimentacoes;
        }

 public async Task<decimal> GetSaldoAtualAsync(int metalId)
        {
            await InitAsync();
   var ultimaMovimentacao = await _database!.Table<MovimentacaoEstoque>()
           .Where(m => m.MetalId == metalId)
    .OrderByDescending(m => m.Id)
     .FirstOrDefaultAsync();

            return ultimaMovimentacao?.SaldoAtual ?? 0;
        }

        public async Task<List<EstoqueAtual>> GetEstoqueAtualAsync()
        {
            await InitAsync();
  var metais = await GetMetaisAsync();
 var estoqueAtual = new List<EstoqueAtual>();

 foreach (var metal in metais)
      {
                var quantidade = await GetSaldoAtualAsync(metal.Id);
      estoqueAtual.Add(new EstoqueAtual
     {
  MetalId = metal.Id,
          NomeMetal = metal.Nome,
           UnidadeMedida = metal.UnidadeMedida,
  QuantidadeAtual = quantidade,
    EstoqueMinimo = metal.EstoqueMinimo,
      PrecoCompraAtual = metal.PrecoCompra,
    PrecoVendaAtual = metal.PrecoVenda,
           ValorEstoque = quantidade * metal.PrecoCompra
        });
   }

      return estoqueAtual.OrderBy(e => e.NomeMetal).ToList();
        }

        // ==================== RELATÓRIOS ====================
        
        public async Task<decimal> GetTotalComprasAsync(DateTime dataInicio, DateTime dataFim)
     {
        await InitAsync();
            var compras = await _database!.Table<Compra>()
           .Where(c => c.DataCompra >= dataInicio && c.DataCompra <= dataFim)
       .ToListAsync();
            
       return compras.Sum(c => c.ValorTotal);
   }

        public async Task<decimal> GetTotalVendasAsync(DateTime dataInicio, DateTime dataFim)
        {
            await InitAsync();
            var vendas = await _database!.Table<Venda>()
        .Where(v => v.DataVenda >= dataInicio && v.DataVenda <= dataFim)
                .ToListAsync();
     
        return vendas.Sum(v => v.ValorTotal);
        }

   public async Task<decimal> GetLucroBrutoAsync(DateTime dataInicio, DateTime dataFim)
        {
  var totalVendas = await GetTotalVendasAsync(dataInicio, dataFim);
    var totalCompras = await GetTotalComprasAsync(dataInicio, dataFim);
       return totalVendas - totalCompras;
        }
    }
}
