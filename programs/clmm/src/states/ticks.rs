use anchor_lang::prelude::*;

use crate::{
    errors::ClmmError,
    utils::{ANCHOR_DISCRIMINATOR, TICKS_PER_ARRAY},
};

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

    pub fn update_liquidity(&mut self, liquidity_delta: i128, is_lower: bool) -> Result<()> {
        if !self.initialized {
            self.initialized = true;
        }

        self.gross_liquidity = self
            .gross_liquidity
            .checked_add(liquidity_delta.unsigned_abs())
            .ok_or(ClmmError::ArithmeticOverflow)?;

        if is_lower {
            self.net_liquidity = self
                .net_liquidity
                .checked_add(liquidity_delta)
                .ok_or(ClmmError::ArithmeticOverflow)?;
        } else {
            self.net_liquidity = self
                .net_liquidity
                .checked_sub(liquidity_delta)
                .ok_or(ClmmError::ArithmeticOverflow)?;
        }

        Ok(())
    }
}

#[account]
pub struct TickArrayState {
    pub pool: Pubkey,
    pub starting_tick: i32,
    pub ticks: [TickState; TICKS_PER_ARRAY],
    pub bump: u8,
}

impl TickArrayState {
    pub const SPACE: usize = ANCHOR_DISCRIMINATOR +
32 + // pool
4 + // starting tick
TICKS_PER_ARRAY * 41 + // one tick is 41 bytes
1; // bump
}

impl TickArrayState {
    // this function finds the start index of the array in which the provided tick is present
    pub fn get_start_tick_idx(tick: i32, tick_spacing: i32) -> i32 {
        let ticks_per_array = TICKS_PER_ARRAY as i32;

        let array_idx = tick
            .checked_div(tick_spacing)
            .expect("Tick spacing: division by 0")
            .checked_div(ticks_per_array)
            .expect("Ticks per array: division by 0");

        let start_tick = array_idx
            .checked_mul(tick_spacing)
            .expect("tick spacing: multiplication oveflow")
            .checked_mul(ticks_per_array)
            .expect("ticks per array: multiplication overflow");

        start_tick
    }
// this function finds the tick state of the given tick from the aray and returns it mutably
pub fn get_tick_state_mut(&mut self, tick: i32, tick_spacing: i32)
->Result<&mut TickState>{
    let ticks_per_array = TICKS_PER_ARRAY as i32;

    let offset = (tick
    .checked_div(tick_spacing) // position of the tick in the sequence
    .ok_or(ClmmError::ArithmeticOverflow)?)
    .checked_sub(self.starting_tick 
    .checked_div(tick_spacing)// distance from the first idx
    .ok_or(ClmmError::ArithmeticOverflow)?,)
    .ok_or(ClmmError::ArithmeticOverflow)?
    .checked_rem(ticks_per_array) // to make sure offset stays in the range
    .ok_or(ClmmError::ArithmeticOverflow)? as usize;

Ok(&mut self.ticks[offset])

}
}
