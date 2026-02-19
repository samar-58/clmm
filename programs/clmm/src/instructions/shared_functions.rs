use anchor_lang::prelude::*;
use anchor_spl::token_interface::{
    transfer_checked, Mint, TokenAccount, TokenInterface, TransferChecked,
};

use crate::states::Pool;

pub fn transfer_tokens<'info>(
    from: &InterfaceAccount<'info, TokenAccount>,
    to: &InterfaceAccount<'info, TokenAccount>,
    amount: &u64,
    mint: &InterfaceAccount<'info, Mint>,
    authority: &Signer<'info>,
    token_program: &Interface<'info, TokenInterface>,
) -> Result<()> {
    let transfer_account_options = TransferChecked {
        from: from.to_account_info(),
        mint: mint.to_account_info(),
        to: to.to_account_info(),
        authority: authority.to_account_info(),
    };

    let cpi_context = CpiContext::new(token_program.to_account_info(), transfer_account_options);

    transfer_checked(cpi_context, *amount, mint.decimals)
}

pub fn transfer_from_pda<'info>(
    from: &InterfaceAccount<'info, TokenAccount>,
    to: &InterfaceAccount<'info, TokenAccount>,
    amount: &u64,
    mint: &InterfaceAccount<'info, Mint>,
    token_program: &Interface<'info, TokenInterface>,
    pool: &Account<'info, Pool>,
) -> Result<()> {
    let transfer_account_options = TransferChecked {
        from: from.to_account_info(),
        mint: mint.to_account_info(),
        to: to.to_account_info(),
        authority: pool.to_account_info(),
    };

    let seeds: &[&[u8]] = &[
        b"pool",
        pool.token_0.as_ref(),
        pool.token_1.as_ref(),
        &pool.tick_spacing.to_le_bytes(),
        &[pool.bump],
    ];

    let signer_seeds = &[seeds];
    let cpi_ctx = CpiContext::new_with_signer(
        token_program.to_account_info(),
        transfer_account_options,
        signer_seeds,
    );
    transfer_checked(cpi_ctx, *amount, mint.decimals)
}
