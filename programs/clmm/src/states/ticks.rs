use anchor_lang::prelude::*;

use crate::{errors::ClmmError, utils::ANCHOR_DISCRIMINATOR};

#[account]
pub struct TickState {
    pub initialized: bool,
    pub gross_liquidity: u128,
    pub net_liquidity: i128,
}
impl TickState {
    pub const SPACE: usize = ANCHOR_DISCRIMINATOR +
1 + // initialized
16 + // gross liquidity
16; // net liquidity

pub fn update_liquidity(&mut self, liquidity_delta: i128, is_lower: bool)->Result<()>{

if !self.initialized{
    self.initialized = true;
}

self.gross_liquidity = self.gross_liquidity.checked_add(liquidity_delta.unsigned_abs()).ok_or(ClmmError::ArithmeticOverflow)?;

if is_lower{
    self.net_liquidity = self.net_liquidity.checked_add(liquidity_delta).ok_or(ClmmError::ArithmeticOverflow)?;
}
else{
    self.net_liquidity = self.net_liquidity.checked_sub(liquidity_delta).ok_or(ClmmError::ArithmeticOverflow)?;
}

    Ok(())
}

}
