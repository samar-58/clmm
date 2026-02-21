use anchor_lang::prelude::*;
use anchor_spl::token_interface::{Mint, TokenAccount, TokenInterface};

use crate::{
    errors::ClmmError,
    instructions::transfer_from_pda,
    states::{Pool, Position, TickArrayState},
    utils::*,
};

#[derive(Accounts)]
#[instruction(upper_tick: i32, lower_tick: i32, tick_array_lower_start_index:i32, tick_array_upper_start_index:i32)]
pub struct ClosePosition<'info> {
    #[account(mut)]
    pub signer: Signer<'info>,
    #[account(
    mut,
    has_one = token_0,
    has_one = token_1
)]
    pub pool: Box<Account<'info, Pool>>,

    pub token_0: Box<InterfaceAccount<'info, Mint>>,
    pub token_1: Box<InterfaceAccount<'info, Mint>>,

    #[account(
    mut,
    seeds = [
        b"tick_array",
        pool.key().as_ref(),
        &tick_array_lower_start_index.to_le_bytes()
    ],
    bump
)]
    pub lower_tick_array: Box<Account<'info, TickArrayState>>,
    #[account(
    mut,
    seeds = [
        b"tick_array",
        pool.key().as_ref(),
        tick_array_upper_start_index.to_le_bytes().as_ref()
    ],
    bump
)]
    pub upper_tick_array: Box<Account<'info, TickArrayState>>,

    #[account(
    mut,
    close = signer,
    seeds = [
        b"position",
        pool.key().as_ref(),
        signer.key().as_ref(),
        lower_tick.to_le_bytes().as_ref(),
        upper_tick.to_le_bytes().as_ref()
    ],
    bump
)]
    pub position: Box<Account<'info, Position>>,
    #[account(mut)]
    pub user_0: Box<InterfaceAccount<'info, TokenAccount>>,
    #[account(mut)]
    pub user_1: Box<InterfaceAccount<'info, TokenAccount>>,

    #[account(
    mut,
    token::mint = token_0,
    token::authority = pool
)]
    pub pool_vault_0: Box<InterfaceAccount<'info, TokenAccount>>,
    #[account(
    mut,
    token::mint = token_1,
    token::authority = pool
)]
    pub pool_vault_1: Box<InterfaceAccount<'info, TokenAccount>>,

    pub system_program: Program<'info, System>,
    pub token_program: Interface<'info, TokenInterface>,
    pub rent: Sysvar<'info, Rent>,
}

pub fn close_position(
    ctx: Context<ClosePosition>,
    upper_tick: i32,
    lower_tick: i32,
) -> Result<(u64, u64)> {
    let pool = &mut ctx.accounts.pool;
    let position = &mut ctx.accounts.position;
    let liquidity_to_remove = position.liquidity;

    let position_lower_tick = position.lower_tick;
    let position_upper_tick = position.upper_tick;
    require!(
        upper_tick == position_upper_tick && lower_tick == position_lower_tick,
        ClmmError::ZeroAmount
    );
    let upper_tick_array = &mut ctx.accounts.upper_tick_array;
    let lower_tick_array = &mut ctx.accounts.lower_tick_array;

    let lower_tick_state = lower_tick_array.get_tick_state_mut(lower_tick, pool.tick_spacing)?;

    let upper_tick_state = upper_tick_array.get_tick_state_mut(upper_tick, pool.tick_spacing)?;

    lower_tick_state.update_liquidity(-(liquidity_to_remove as i128), true)?;
    upper_tick_state.update_liquidity(-(liquidity_to_remove as i128), false)?;

    let lower_sqrt = tick_to_sqrt_price_x96(lower_tick)?;
    let upper_sqrt = tick_to_sqrt_price_x96(upper_tick)?;

    if pool.sqrt_price_x96 >= lower_sqrt && pool.sqrt_price_x96 < upper_sqrt {
        pool.global_liquidity = pool
            .global_liquidity
            .checked_sub(liquidity_to_remove)
            .ok_or(ClmmError::ArithmeticOverflow)?;
    }

    let (amount_0, amount_1) = get_amounts_for_liquidity(
        pool.sqrt_price_x96,
        lower_sqrt,
        upper_sqrt,
        liquidity_to_remove,
    )?;

    if amount_0 > 0 {
        transfer_from_pda(
            &ctx.accounts.pool_vault_0,
            &ctx.accounts.user_0,
            &amount_0,
            &ctx.accounts.token_0,
            &ctx.accounts.token_program,
            &ctx.accounts.pool,
        )?;
    }

    if amount_1 > 0 {
        transfer_from_pda(
            &ctx.accounts.pool_vault_1,
            &ctx.accounts.user_1,
            &amount_1,
            &ctx.accounts.token_1,
            &ctx.accounts.token_program,
            &ctx.accounts.pool,
        )?;
    }

    Ok((amount_0, amount_1))
}
