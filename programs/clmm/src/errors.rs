use anchor_lang::prelude::*;

#[error_code]
pub enum ClmmError {
    #[msg("Arithmetic Overflow")]
    ArithmeticOverflow,
    #[msg("Invalid Token Pair")]
    InvalidTokenPair,
    #[msg("Invalid Token Pair")]
    InvalidTickSpacing,
}
