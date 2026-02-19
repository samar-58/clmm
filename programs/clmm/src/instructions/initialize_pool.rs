use anchor_lang::prelude::*;
use anchor_spl::token_interface::{Mint, TokenAccount, TokenInterface};

use crate::{errors::ClmmError, states::Pool, utils::sqrt_price_x96_to_tick};
#[derive(Accounts)]
#[instruction(tick_spacing: i32)]
pub struct InitializePool<'info> {
    #[account(mut)]
    pub signer: Signer<'info>,

    pub token_0_mint: InterfaceAccount<'info, Mint>,
    pub token_1_mint: InterfaceAccount<'info, Mint>,

    #[account(
init,
payer = signer,
space = Pool::SPACE,
seeds = [
b"pool".as_ref(),
token_0_mint.key().as_ref(),
token_1_mint.key().as_ref(),
tick_spacing.to_le_bytes().as_ref()
],
bump
)]
    pub pool: Account<'info, Pool>,

    #[account(
    init,
    payer = signer,
    token::mint = token_0_mint,
    token::authority = pool
)]
    pub token_0_vault: InterfaceAccount<'info, TokenAccount>,

    #[account(
    init,
    payer = signer,
    token::mint = token_1_mint,
    token::authority = pool
)]
    pub token_1_vault: InterfaceAccount<'info, TokenAccount>,

    pub system_program: Program<'info, System>,
    pub token_program: Interface<'info, TokenInterface>,
    pub rent: Sysvar<'info, Rent>,
}

pub fn init_pool(
    ctx: Context<InitializePool>,
    tick_spacing: i32,
    initialize_sqrt_price: u128,
) -> Result<()> {
    require!(
        ctx.accounts.token_0_mint.key() < ctx.accounts.token_1_mint.key(),
        ClmmError::InvalidTokenOrder
    );
    require!(tick_spacing > 0, ClmmError::InvalidTickSpacing);
    require!(
        ctx.accounts.token_0_mint.key() != ctx.accounts.token_1_mint.key(),
        ClmmError::InvalidTokenPair
    );
    let pool = &mut ctx.accounts.pool;
    pool.set_inner(Pool {
        token_0: ctx.accounts.token_0_mint.key(),
        token_1: ctx.accounts.token_1_mint.key(),
        token_vault_0: ctx.accounts.token_0_vault.key(),
        token_vault_1: ctx.accounts.token_1_vault.key(),
        global_liquidity: 0,
        sqrt_price_x96: initialize_sqrt_price,
        current_tick: sqrt_price_x96_to_tick(initialize_sqrt_price)?,
        tick_spacing,
        bump: ctx.bumps.pool,
    });

    Ok(())
}
