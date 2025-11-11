using SQLite;

namespace Controle_Roncatin.Models
{
    /// <summary>
    /// Representa uma venda de sucata para indústria
    /// </summary>
    public class Venda
    {
        [PrimaryKey, AutoIncrement]
    public int Id { get; set; }

        /// <summary>
  /// ID do metal vendido
        /// </summary>
    [Indexed, NotNull]
        public int MetalId { get; set; }

        /// <summary>
        /// Nome da empresa/indústria compradora
        /// </summary>
        [MaxLength(200), NotNull]
        public string NomeCliente { get; set; } = string.Empty;

        /// <summary>
        /// CNPJ da empresa
      /// </summary>
        [MaxLength(20)]
        public string? CnpjCliente { get; set; }

      /// <summary>
        /// Telefone do cliente
        /// </summary>
        [MaxLength(20)]
  public string? TelefoneCliente { get; set; }

        /// <summary>
        /// Email do cliente
        /// </summary>
        [MaxLength(100)]
        public string? EmailCliente { get; set; }

        /// <summary>
        /// Quantidade/Peso vendido (em KG, TON, etc)
      /// </summary>
        [NotNull]
    public decimal Quantidade { get; set; }

        /// <summary>
        /// Preço unitário de venda
      /// </summary>
   [NotNull]
        public decimal PrecoUnitario { get; set; }

        /// <summary>
 /// Valor total da venda
     /// </summary>
        [NotNull]
        public decimal ValorTotal { get; set; }

 /// <summary>
        /// Data e hora da venda
        /// </summary>
        [NotNull]
        public DateTime DataVenda { get; set; } = DateTime.Now;

  /// <summary>
    /// Número da nota fiscal (se houver)
     /// </summary>
        [MaxLength(50)]
        public string? NumeroNF { get; set; }

        /// <summary>
  /// Observações sobre a venda
        /// </summary>
        [MaxLength(500)]
   public string? Observacoes { get; set; }

   /// <summary>
        /// Status do pagamento (Pendente, Pago, Cancelado)
    /// </summary>
        [MaxLength(20), NotNull]
        public string StatusPagamento { get; set; } = "Pendente";

        /// <summary>
        /// Referência ao metal (não persiste no banco)
  /// </summary>
        [Ignore]
        public Metal? Metal { get; set; }
    }
}
