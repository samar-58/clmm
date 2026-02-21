use anchor_lang::prelude::*;
use anchor_spl::token_interface::{Mint, TokenAccount, TokenInterface};

use crate::{
    errors::ClmmError,
    instructions::{transfer_from_pda, transfer_tokens},
    states::{Pool, TickArrayState},
    utils::{compute_swap_step, sqrt_price_x96_to_tick, tick_to_sqrt_price_x96},
};

#[derive(Accounts)]
pub struct Swap<'info> {
    #[account(mut)]
    pub signer: Signer<'info>,

    #[account(
        mut,
        has_one = token_0,
        has_one = token_1
    )]
    pub pool: Box<Account<'info, Pool>>,

    #[account(
        mut,
        constraint = tick_array.key() == Pubkey::find_program_address(
            &[
                b"tick_array",
                pool.key().as_ref(),
                &TickArrayState::get_start_tick_idx(pool.current_tick, pool.tick_spacing).to_le_bytes()
            ],
            &crate::ID
        ).0 @ ClmmError::InvalidTickArrayAccount
    )]
    pub tick_array: Box<Account<'info, TickArrayState>>,

    #[account(
        mut,
        token::mint = token_0,
        token::authority = signer
    )]
    pub user_0: Box<InterfaceAccount<'info, TokenAccount>>,

    #[account(
        mut,
        token::mint = token_1,
        token::authority = signer
    )]
    pub user_1: Box<InterfaceAccount<'info, TokenAccount>>,

    #[account(
        mut,
        token::mint = token_0,
        token::authority = pool
    )]
    pub token_vault_0: Box<InterfaceAccount<'info, TokenAccount>>,

    #[account(
        mut,
        token::mint = token_1,
        token::authority = pool
    )]
    pub token_vault_1: Box<InterfaceAccount<'info, TokenAccount>>,

    pub token_0: Box<InterfaceAccount<'info, Mint>>,
    pub token_1: Box<InterfaceAccount<'info, Mint>>,

    pub system_program: Program<'info, System>,
    pub token_program: Interface<'info, TokenInterface>,
    pub rent: Sysvar<'info, Rent>,
}

pub fn swap(
    ctx: Context<Swap>,
    amount_in: u64,
    a_to_b: bool,
    min_amount_out: u64,
) -> Result<()> {

    let (amount_in_consumed, amount_out, new_tick) = {
        let pool = &mut ctx.accounts.pool;

        require!(amount_in > 0, ClmmError::ZeroAmount);
        require!(pool.global_liquidity > 0, ClmmError::InsufficientLiquidity);

        // when swapping a_to_b:
        //  price goes DOWN, target is the next LOWER tick boundary
        //  next_tick = current_tick rounded down to tick_spacing
        //
        // when swapping b_to_a:
        //  price goes UP, target is the next UPPER tick boundary
        //  next_tick = current_tick rounded up to tick_spacing
        //
        let target_tick = if a_to_b {
            // example: current_tick = 57, spacing = 10 → target = 50
            let tick = (pool.current_tick / pool.tick_spacing) * pool.tick_spacing;
            // if we are exactly on a tick boundary, go one spacing lower
            if tick == pool.current_tick {
                tick - pool.tick_spacing
            } else {
                tick
            }
        } else {
            // example: current_tick = 57, spacing = 10 → target = 60
            ((pool.current_tick / pool.tick_spacing) + 1) * pool.tick_spacing
        };

        let sqrt_price_target_x96 = tick_to_sqrt_price_x96(target_tick)?;

        //  figures out if we can reach the target tick or stop partway
        //  calculates exact input consumed and output produced
        //
        let (next_sqrt_price_x96, amount_consumed, amount_produced) = compute_swap_step(
            pool.sqrt_price_x96,
            sqrt_price_target_x96,
            pool.global_liquidity,
            amount_in as u128,
            a_to_b,
        )?;

        // update pool state
        // after the swap, the price has moved, so we update both
        pool.sqrt_price_x96 = next_sqrt_price_x96;
        pool.current_tick = sqrt_price_x96_to_tick(next_sqrt_price_x96)?;

        // check slippage
        let amount_out: u64 = amount_produced
            .try_into()
            .map_err(|_| ClmmError::ArithmeticOverflow)?;
        let amount_in_consumed: u64 = amount_consumed
            .try_into()
            .map_err(|_| ClmmError::ArithmeticOverflow)?;

        require!(amount_out >= min_amount_out, ClmmError::SlippageExceeded);

        let tick = pool.current_tick;
        (amount_in_consumed, amount_out, tick)
    };
    if a_to_b {
        if amount_in_consumed > 0 {
            transfer_tokens(
                &ctx.accounts.user_0,
                &ctx.accounts.token_vault_0,
                &amount_in_consumed,
                &ctx.accounts.token_0,
                &ctx.accounts.signer,
                &ctx.accounts.token_program,
            )?;
        }

        if amount_out > 0 {
            transfer_from_pda(
                &ctx.accounts.token_vault_1,
                &ctx.accounts.user_1,
                &amount_out,
                &ctx.accounts.token_1,
                &ctx.accounts.token_program,
                &ctx.accounts.pool,
            )?;
        }
    } else {
        if amount_in_consumed > 0 {
            transfer_tokens(
                &ctx.accounts.user_1,
                &ctx.accounts.token_vault_1,
                &amount_in_consumed,
                &ctx.accounts.token_1,
                &ctx.accounts.signer,
                &ctx.accounts.token_program,
            )?;
        }

        if amount_out > 0 {
            transfer_from_pda(
                &ctx.accounts.token_vault_0,
                &ctx.accounts.user_0,
                &amount_out,
                &ctx.accounts.token_0,
                &ctx.accounts.token_program,
                &ctx.accounts.pool,
            )?;
        }
    }

    msg!(
        "Swap: a_to_b={}, in={}, out={}, new_tick={}",
        a_to_b,
        amount_in_consumed,
        amount_out,
        new_tick
    );

    Ok(())
}