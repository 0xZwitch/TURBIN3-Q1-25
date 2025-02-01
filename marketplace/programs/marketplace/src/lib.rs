use anchor_lang::prelude::*;
pub mod contexts;
pub mod states;

pub use contexts::*;
pub use states::*;

declare_id!("4M7ujixxT2BXEPfaTDsCQH4AY547MmYEz6R9WAtZX7dv");

#[program]
pub mod marketplace {
    use super::*;

    pub fn initialize(ctx: Context<Initialize>) -> Result<()> {
        msg!("Greetings from: {:?}", ctx.program_id);
        Ok(())
    }
}

#[derive(Accounts)]
pub struct Initialize {}
