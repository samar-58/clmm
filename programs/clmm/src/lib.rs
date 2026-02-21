use anchor_lang::prelude::*;
pub mod errors;
pub mod instructions;
pub mod states;
pub mod utils;

use instructions::*;

declare_id!("HvVeBmuPRReNPaMXXVWsz8UmtMSbUXnkGoDNN57brQcH");

#[program]
pub mod clmm {
    use super::*;


    pub fn initialize_pool(
        ctx: Context<InitializePool>,
        tick_spacing: i32,
        initialize_sqrt_price: u128,
    ) -> Result<()> {
        instructions::initialize_pool::init_pool(ctx, tick_spacing, initialize_sqrt_price)
    }

    pub fn open_position(
        ctx: Context<OpenPosition>,
        upper_tick: i32,
        lower_tick: i32,
        tick_array_lower_start_index: i32,
        tick_array_upper_start_index: i32,
        liquidity_amount: u128,
    ) -> Result<(u64, u64)> {
        instructions::open_position::open_position(
            ctx,
            upper_tick,
            lower_tick,
            tick_array_lower_start_index,
            tick_array_upper_start_index,
            liquidity_amount,
        )
    }

    pub fn swap(
        ctx: Context<Swap>,
        amount_in: u64,
        a_to_b: bool,
        min_amount_out: u64,
    ) -> Result<()> {
        instructions::swap::swap(ctx, amount_in, a_to_b, min_amount_out)
    }

    pub fn increase_liquidity(
        ctx: Context<IncreaseLiquidity>,
        liquidity_amount: u128,
        upper_tick: i32,
        lower_tick: i32,
        tick_array_lower_start_index: i32,
        tick_array_upper_start_index: i32,
    ) -> Result<()> {
        instructions::increase_liquidity::increase_liquidity(
            ctx,
            liquidity_amount,
            upper_tick,
            lower_tick,
            tick_array_lower_start_index,
            tick_array_upper_start_index,
        )
    }

    pub fn decrease_liquidity(
        ctx: Context<DecreaseLiquidity>,
        liquidity_amount: u128,
        upper_tick: i32,
        lower_tick: i32,
        tick_array_lower_start_index: i32,
        tick_array_upper_start_index: i32,
    ) -> Result<()> {
        instructions::decrease_liquidity::decrease_liquidity(
            ctx,
            liquidity_amount,
            upper_tick,
            lower_tick,
            tick_array_lower_start_index,
            tick_array_upper_start_index,
        )
    }

    pub fn close_position(
        ctx: Context<ClosePosition>,
        upper_tick: i32,
        lower_tick: i32,
        _tick_array_lower_start_index: i32,
        _tick_array_upper_start_index: i32,
    ) -> Result<(u64, u64)> {
        instructions::close_position::close_position(ctx, upper_tick, lower_tick)
    }
}
