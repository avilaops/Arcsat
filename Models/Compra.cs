using SQLite;

namespace Controle_Roncatin.Models
{
    /// <summary>
    /// Representa uma compra de sucata de um catador/trabalhador
    /// </summary>
    public class Compra
    {
        [PrimaryKey, AutoIncrement]
        public int Id { get; set; }

        /// <summary>
        /// ID do metal comprado
        /// </summary>
        [Indexed, NotNull]
        public int MetalId { get; set; }

        /// <summary>
/// Nome do catador/fornecedor
        /// </summary>
        [MaxLength(150), NotNull]
        public string NomeFornecedor { get; set; } = string.Empty;

    /// <summary>
        /// Documento do fornecedor (CPF/RG)
        /// </summary>
      [MaxLength(20)]
        public string? DocumentoFornecedor { get; set; }

        /// <summary>
        /// Telefone do fornecedor
        /// </summary>
        [MaxLength(20)]
        public string? TelefoneFornecedor { get; set; }

        /// <summary>
        /// Quantidade/Peso comprado (em KG, TON, etc)
 /// </summary>
        [NotNull]
        public decimal Quantidade { get; set; }

        /// <summary>
        /// Preço unitário pago
        /// </summary>
        [NotNull]
    public decimal PrecoUnitario { get; set; }

        /// <summary>
        /// Valor total da compra
   /// </summary>
        [NotNull]
        public decimal ValorTotal { get; set; }

        /// <summary>
        /// Data e hora da compra
        /// </summary>
    [NotNull]
        public DateTime DataCompra { get; set; } = DateTime.Now;

/// <summary>
        /// Observações sobre a compra
        /// </summary>
        [MaxLength(500)]
        public string? Observacoes { get; set; }

        /// <summary>
      /// Referência ao metal (não persiste no banco)
        /// </summary>
  [Ignore]
        public Metal? Metal { get; set; }
    }
}
