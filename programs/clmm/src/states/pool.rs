use anchor_lang::prelude::*;
use crate::utils::ANCHOR_DISCRIMINATOR;

#[account]
#[derive(InitSpace)]
pub struct Pool {
    pub token_0: Pubkey,
    pub token_1: Pubkey,
    pub token_vault_0: Pubkey,
    pub token_vault_1: Pubkey,
    pub global_liquidity: u128,
    //ratio of price * 2**96
    pub sqrt_price_x96: u128,
    //formula of tick
    //current_tick = log(price)/log(1.0001) as price = 1.0001**tick
    pub current_tick: i32,
    //the interval between ticks which will allow lps to choose their lower and upper positions
    pub tick_spacing: i32,
    pub bump: u8,
}

impl Pool {
    pub const SPACE: usize = ANCHOR_DISCRIMINATOR +
    32 + // token_0 pubkey 
    32 + // token_1 pubkey
    32 + // token_0_vault pubkey
    32 + // token_1_vault pubkey
    16 + // global_liquidity
    16 + // sqrt_price_x64
    4 +  // current_tick
    4 +  // tick_spacing
    1; // bump
}
