use anchor_lang::prelude::*;
pub mod errors;
pub mod instructions;
pub mod states;
pub mod utils;
declare_id!("HvVeBmuPRReNPaMXXVWsz8UmtMSbUXnkGoDNN57brQcH");

#[program]
pub mod clmm {
    use super::*;

    pub fn initialize(ctx: Context<Initialize>) -> Result<()> {
        msg!("Greetings from: {:?}", ctx.program_id);
        Ok(())
    }
}

#[derive(Accounts)]
pub struct Initialize {}
