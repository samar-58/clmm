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
    SqrtPriceX96,
    #[msg("Invalid Position Range")]
    InvalidPositionRange,
    #[msg("Invalid Position Owner")]
    InvalidPositionOwner,
    #[msg("Invalid Mint Range")]
    InvalidMintRange,
    #[msg("Invalid Tick Range")]
    InvalidTickRange,
    #[msg("Invalid Ticks Provided")]
    InvalidTicks,
    #[msg("Invalid Amount Entered")]
    InvalidAmount,
}
