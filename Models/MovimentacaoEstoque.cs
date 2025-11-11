using SQLite;

namespace Controle_Roncatin.Models
{
    /// <summary>
    /// Representa o estoque atual de cada tipo de metal
    /// </summary>
    public class MovimentacaoEstoque
  {
        [PrimaryKey, AutoIncrement]
        public int Id { get; set; }

        /// <summary>
        /// ID do metal
        /// </summary>
  [Indexed, NotNull]
        public int MetalId { get; set; }

        /// <summary>
  /// Tipo de movimentação (COMPRA, VENDA, AJUSTE)
     /// </summary>
  [MaxLength(20), NotNull]
public string TipoMovimentacao { get; set; } = string.Empty;

        /// <summary>
 /// Quantidade movimentada (positivo = entrada, negativo = saída)
 /// </summary>
  [NotNull]
public decimal Quantidade { get; set; }

        /// <summary>
   /// Saldo anterior à movimentação
        /// </summary>
    public decimal SaldoAnterior { get; set; }

     /// <summary>
        /// Saldo após a movimentação
  /// </summary>
      public decimal SaldoAtual { get; set; }

        /// <summary>
    /// ID da compra relacionada (se aplicável)
        /// </summary>
    [Indexed]
        public int? CompraId { get; set; }

        /// <summary>
        /// ID da venda relacionada (se aplicável)
        /// </summary>
        [Indexed]
        public int? VendaId { get; set; }

        /// <summary>
        /// Data e hora da movimentação
        /// </summary>
  [NotNull]
        public DateTime DataMovimentacao { get; set; } = DateTime.Now;

        /// <summary>
        /// Observações sobre a movimentação
        /// </summary>
        [MaxLength(300)]
     public string? Observacoes { get; set; }

        /// <summary>
        /// Referência ao metal (não persiste no banco)
        /// </summary>
        [Ignore]
        public Metal? Metal { get; set; }
    }

    /// <summary>
    /// View consolidada do estoque por metal
    /// </summary>
    public class EstoqueAtual
    {
        public int MetalId { get; set; }
        public string NomeMetal { get; set; } = string.Empty;
     public string UnidadeMedida { get; set; } = string.Empty;
   public decimal QuantidadeAtual { get; set; }
        public decimal EstoqueMinimo { get; set; }
        public decimal PrecoCompraAtual { get; set; }
        public decimal PrecoVendaAtual { get; set; }
        public decimal ValorEstoque { get; set; }
        public bool AbaixoMinimo => QuantidadeAtual < EstoqueMinimo;
    }
}
