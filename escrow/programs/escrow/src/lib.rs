pub mod instructions;
pub mod states;

use anchor_lang::prelude::*;

pub use instructions::*;
pub use states::*;

declare_id!("7F83HDdhKh85GArEurYnZwdN9gLkNGDnkefwDiZphkjC");

#[program]
pub mod escrow {
    use super::*;

    pub fn make(
        ctx: Context<Make>,
        seed: u64,
        receive_amount: u64,
        deposit_amount: u64,
    ) -> Result<()> {
        ctx.accounts
            .init_escrow_state(seed, receive_amount, ctx.bumps)?;
        ctx.accounts.deposit(deposit_amount)?;
        Ok(())
    }

    pub fn take(ctx: Context<Take>) -> Result<()> {
        ctx.accounts.withdraw()?;
        ctx.accounts.close()?;
        Ok(())
    }

    pub fn refund(ctx: Context<Refund>) -> Result<()> {
        ctx.accounts.withdraw()?;
        ctx.accounts.close()?;
        Ok(())
    }
}
