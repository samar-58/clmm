use anchor_lang::prelude::*;

#[account]
#[derive(InitSpace)]
pub struct Pool {
    pub token_0: Pubkey,
    pub token_1: Pubkey,
    pub token_vault_0: Pubkey,
    pub token_vault_1: Pubkey,
    pub global_liquidity: u128,
    pub sqrt_price_x64: u128,
    pub current_tick: i32,
    pub tick_spacing: i32,
    pub bump: u8,
}

impl Pool {
    pub const SPACE: usize = 8 + // discriminator
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
