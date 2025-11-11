using SQLite;

namespace Controle_Roncatin.Models
{
  /// <summary>
    /// Representa um tipo de metal comercializado (ex: Alumínio, Cobre, Ferro, etc.)
    /// </summary>
    public class Metal
    {
 [PrimaryKey, AutoIncrement]
        public int Id { get; set; }

    /// <summary>
        /// Nome do metal (ex: Alumínio, Cobre, Ferro, Aço, Bronze, etc.)
   /// </summary>
        [MaxLength(100), NotNull]
        public string Nome { get; set; } = string.Empty;

        /// <summary>
  /// Descrição ou categoria do metal (ex: "Lata de refrigerante", "Fio de cobre")
        /// </summary>
        [MaxLength(250)]
        public string? Descricao { get; set; }

        /// <summary>
        /// Unidade de medida (KG, TON, UNIDADE)
        /// </summary>
        [MaxLength(10), NotNull]
        public string UnidadeMedida { get; set; } = "KG";

 /// <summary>
        /// Preço de compra por unidade (quanto você paga ao catador)
        /// </summary>
   public decimal PrecoCompra { get; set; }

      /// <summary>
        /// Preço de venda por unidade (quanto você vende para indústria)
        /// </summary>
        public decimal PrecoVenda { get; set; }

   /// <summary>
     /// Quantidade mínima em estoque para alerta
        /// </summary>
   public decimal EstoqueMinimo { get; set; }

        /// <summary>
   /// Indica se o metal está ativo para comercialização
    /// </summary>
        public bool Ativo { get; set; } = true;

        /// <summary>
        /// Data de cadastro
      /// </summary>
        public DateTime DataCadastro { get; set; } = DateTime.Now;

        /// <summary>
   /// Calcula a margem de lucro percentual
        /// </summary>
   [Ignore]
        public decimal MargemLucro => PrecoCompra > 0 ? ((PrecoVenda - PrecoCompra) / PrecoCompra) * 100 : 0;

  /// <summary>
        /// Calcula o lucro unitário
        /// </summary>
        [Ignore]
        public decimal LucroUnitario => PrecoVenda - PrecoCompra;
  }
}
