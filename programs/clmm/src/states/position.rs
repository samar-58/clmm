use anchor_lang::prelude::*;
use crate::utils::ANCHOR_DISCRIMINATOR;


#[account]
pub struct Position{
pub liquidity: u128,
pub lower_tick: i32,
pub upper_tick: i32,
pub owner: Pubkey,
pub pool: Pubkey,
pub bump: u8
}

impl Position{
pub const SPACE: usize = ANCHOR_DISCRIMINATOR +
16 +  // liquidity
4 + // lower_tick
4 + // upper_tick
32 + // owner
32 + // pool
1; // bump
}