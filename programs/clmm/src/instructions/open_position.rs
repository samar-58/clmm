use anchor_lang::prelude::*;
use anchor_spl::token_interface::{Mint, TokenAccount, TokenInterface};

use crate::{
    errors::ClmmError,
    instructions::transfer_tokens,
    states::{Pool, Position, TickArrayState},
    utils::*,
};

#[derive(Accounts)]
#[instruction(upper_tick: i32, lower_tick: i32, tick_array_lower_start_index:i32, tick_array_upper_start_index:i32)]
pub struct OpenPosition<'info> {
    #[account(mut)]
    pub signer: Signer<'info>,
    #[account(
    mut,
    has_one = token_0,
    has_one = token_1
)]
    pub pool: Account<'info, Pool>,

    pub token_0: InterfaceAccount<'info, Mint>,
    pub token_1: InterfaceAccount<'info, Mint>,

    #[account(
    init_if_needed,
    payer = signer,
    space = TickArrayState::SPACE,
    seeds = [
        b"tick_array",
        pool.key().as_ref(),
        &tick_array_lower_start_index.to_le_bytes()
    ],
    bump
)]
    pub lower_tick_array: Account<'info, TickArrayState>,
    #[account(
    init_if_needed,
    payer = signer,
    space = TickArrayState::SPACE,
    seeds = [
        b"tick_array",
        pool.key().as_ref(),
        tick_array_upper_start_index.to_le_bytes().as_ref()
    ],
    bump
)]
    pub upper_tick_array: Account<'info, TickArrayState>,

    #[account(
    init,
    payer = signer,
    space = Position::SPACE,
    seeds = [
        b"position",
        pool.key().as_ref(),
        signer.key().as_ref(),
        lower_tick.to_le_bytes().as_ref(),
        upper_tick.to_le_bytes().as_ref()
    ],
    bump
)]
    pub position: Account<'info, Position>,
    #[account(mut)]
    pub user_0: InterfaceAccount<'info, TokenAccount>,
    #[account(mut)]
    pub user_1: InterfaceAccount<'info, TokenAccount>,

    #[account(
    mut,
    token::mint = token_0,
    token::authority = pool
)]
    pub pool_vault_0: InterfaceAccount<'info, TokenAccount>,
    #[account(
    mut,
    token::mint = token_1,
    token::authority = pool
)]
    pub pool_vault_1: InterfaceAccount<'info, TokenAccount>,

    pub system_program: Program<'info, System>,
    pub token_program: Interface<'info, TokenInterface>,
    pub rent: Sysvar<'info, Rent>,
}

pub fn open_position(
    ctx: Context<OpenPosition>,
    upper_tick: i32,
    lower_tick: i32,
    tick_array_lower_start_index: i32,
    tick_array_upper_start_index: i32,
    liquidity_amount: u128,
) -> Result<(u64, u64)> {
    let pool = &mut ctx.accounts.pool;
    let position = &mut ctx.accounts.position;

    require!(liquidity_amount > 0, ClmmError::ZeroAmount);

    let upper_tick_array = &mut ctx.accounts.upper_tick_array;
    let lower_tick_array = &mut ctx.accounts.lower_tick_array;

    if lower_tick_array.starting_tick == 0 && lower_tick_array.pool == Pubkey::default() {
        lower_tick_array.pool = pool.key();
        lower_tick_array.starting_tick = tick_array_lower_start_index;
    }

    if upper_tick_array.starting_tick == 0 && upper_tick_array.pool == Pubkey::default() {
        upper_tick_array.pool = pool.key();
        upper_tick_array.starting_tick = tick_array_upper_start_index;
    }

    let lower_tick_state = lower_tick_array.get_tick_state_mut(lower_tick, pool.tick_spacing)?;

    let upper_tick_state = upper_tick_array.get_tick_state_mut(upper_tick, pool.tick_spacing)?;

    lower_tick_state.update_liquidity(liquidity_amount as i128, true)?;
    upper_tick_state.update_liquidity(liquidity_amount as i128, false)?;

    position.set_inner(Position {
        liquidity: liquidity_amount,
        lower_tick,
        upper_tick,
        owner: ctx.accounts.signer.key(),
        pool: pool.key(),
        bump: ctx.bumps.position,
    });

    let lower_sqrt = tick_to_sqrt_price_x96(lower_tick)?;
    let upper_sqrt = tick_to_sqrt_price_x96(upper_tick)?;

    if pool.sqrt_price_x96 >= lower_sqrt && pool.sqrt_price_x96 < upper_sqrt {
        pool.global_liquidity = pool
            .global_liquidity
            .checked_add(liquidity_amount)
            .ok_or(ClmmError::ArithmeticOverflow)?;
    }

    let (amount_0, amount_1) = get_amounts_for_liquidity(
        pool.sqrt_price_x96,
        lower_sqrt,
        upper_sqrt,
        liquidity_amount,
    )?;

    if amount_0 > 0 {
        transfer_tokens(
            &ctx.accounts.user_0,
            &ctx.accounts.pool_vault_0,
            &amount_0,
            &ctx.accounts.token_0,
            &ctx.accounts.signer,
            &ctx.accounts.token_program,
        )?;
    }

    if amount_1 > 0 {
        transfer_tokens(
            &ctx.accounts.user_1,
            &ctx.accounts.pool_vault_1,
            &amount_1,
            &ctx.accounts.token_1,
            &ctx.accounts.signer,
            &ctx.accounts.token_program,
        )?;
    }

    Ok((amount_0, amount_1))
}
