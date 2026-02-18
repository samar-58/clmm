use anchor_lang::prelude::*;

#[error_code]
pub enum ClmmError {
    #[msg("Arithmetic Overflow")]
    ArithmeticOverflow,
    #[msg("Invalid Token Pair")]
    InvalidTokenPair,
    #[msg("Invalid Token Pair")]
    InvalidTickSpacing,
    #[msg("Invalid Token Order")]
    InvalidTokenOrder,
    #[msg("Amount entered is 0")]
    ZeroAmount,
    #[msg("Tick upper over flow")]
    TickUpperOverflow,
    #[msg("Error while sqrt price")]
    SqrtPriceX96
}
