use anchor_lang::prelude::*;
use anchor_spl::token_interface::{Mint, TokenAccount, TokenInterface};

use crate::states::{Pool, Position, TickArrayState};

#[derive(Accounts)]
#[instruction(upper_tick: i32, lower_tick: i32, tick_array_lower_start_index:i32, tick_array_upper_start_index:i32)]
pub struct OpenPosition<'info>{
#[account(mut)]
pub signer: Signer<'info>,
#[account(
    mut,
    has_one = token_0,
    has_one = token_1
)]
pub pool: Account<'info, Pool>,

pub token_0: InterfaceAccount<'info,Mint>,
pub token_1: InterfaceAccount<'info,Mint>,

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

pub user_0: InterfaceAccount<'info, TokenAccount>,
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
pub rent: Sysvar<'info, Rent>
}

pub fn open_position(ctx: Context<OpenPosition>, upper_tick: i32, lower_tick: i32, owner: Pubkey, tick_array_lower_start_index:i32, tick_array_upper_start_index:i32)
->Result<()>{

Ok(())
}