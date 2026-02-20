use anchor_lang::prelude::*;
use anchor_spl::token_interface::{Mint, TokenAccount, TokenInterface};

use crate::{errors::ClmmError, states::{Pool, TickArrayState}};

#[derive(Accounts)]
pub struct Swap<'info>{
    #[account(mut)]
pub signer: Signer<'info>,
#[account(
    mut,
    has_one = token_0,
    has_one = token_1
)]
pub pool: Account<'info, Pool>,

#[account(
    mut,
    constraint = tick_array.key() == Pubkey::find_program_address(
        &[
        b"tick_array",
        pool.key().as_ref(),
        &TickArrayState::get_start_tick_idx(pool.current_tick, pool.tick_spacing).to_le_bytes()
        ],
        &crate::ID).0 @ClmmError::InvalidTickArrayAccount
)]
pub tick_array: Account<'info, TickArrayState>,

#[account(
    init_if_needed,
    payer = signer,
    token::mint = token_0,
    token::authority = signer
)]
pub user_0: InterfaceAccount<'info, TokenAccount>,

#[account(
    init_if_needed,
    payer = signer,
    token::mint = token_0,
    token::authority = signer
)]
pub user_1: InterfaceAccount<'info, TokenAccount>,
#[account(
    mut,
    token::mint = token_0,
    token::authority = signer
)]
pub token_vault_0: InterfaceAccount<'info, TokenAccount>,
pub token_vault_1: InterfaceAccount<'info, TokenAccount>,

pub token_0: InterfaceAccount<'info, Mint>,
pub token_1: InterfaceAccount<'info, Mint>,

pub system_program: Program<'info, System>,
pub token_program: Interface<'info, TokenInterface>,
pub rent: Sysvar<'info, Rent>
}