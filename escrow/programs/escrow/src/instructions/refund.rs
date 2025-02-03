use anchor_lang::prelude::*;
use anchor_spl::{
    associated_token::AssociatedToken,
    token::{close_account, transfer_checked, CloseAccount, TransferChecked},
    token_interface::{Mint, TokenAccount, TokenInterface},
};

use crate::states::*;

#[derive(Accounts)]
pub struct Refund<'info> {
    #[account(mut)]
    pub maker: Signer<'info>,
    pub mint_a: InterfaceAccount<'info, Mint>,
    pub mint_b: InterfaceAccount<'info, Mint>,
    #[account(
    mut,
    associated_token::mint = mint_a,
    associated_token::authority = maker,
  )]
    pub maker_mint_a_ata: InterfaceAccount<'info, TokenAccount>,
    #[account(
    mut,
    close = maker,
    seeds = [b"escrow", maker.key.as_ref(), escrow.seed.to_le_bytes().as_ref()],
    bump = escrow.bump,
  )]
    pub escrow: Account<'info, EscrowState>,
    #[account(
    mut,
    associated_token::mint = mint_a,
    associated_token::authority = escrow,
  )]
    pub vault: InterfaceAccount<'info, TokenAccount>,
    pub system_program: Program<'info, System>,
    pub associated_token_program: Program<'info, AssociatedToken>,
    pub token_program: Interface<'info, TokenInterface>,
}

impl<'info> Refund<'info> {
    pub fn withdraw(&mut self) -> Result<()> {
        let cpi_program = self.token_program.to_account_info();
        let cpi_account = TransferChecked {
            mint: self.mint_a.to_account_info(),
            to: self.maker_mint_a_ata.to_account_info(),
            from: self.vault.to_account_info(),
            authority: self.escrow.to_account_info(),
        };
        let cpi_ctx = CpiContext::new(cpi_program.clone(), cpi_account);

        transfer_checked(cpi_ctx, self.vault.amount, self.mint_a.decimals)?;
        Ok(())
    }

    pub fn close(&mut self) -> Result<()> {
        let cpi_program = self.token_program.to_account_info();

        let cpi_account = CloseAccount {
            authority: self.escrow.to_account_info(),
            account: self.vault.to_account_info(),
            destination: self.maker.to_account_info(),
        };

        let maker_binding = self.escrow.maker.to_bytes();
        let seed_binding = self.escrow.seed.to_le_bytes();
        let bump_binding = self.escrow.bump;

        let seeds: [&[u8]; 4] = [b"escrow", &maker_binding, &seed_binding, &[bump_binding]];
        let signer_seeds: &[&[&[u8]]] = &[&seeds];
        let cpi_ctx = CpiContext::new_with_signer(cpi_program, cpi_account, &signer_seeds);

        close_account(cpi_ctx)?;

        Ok(())
    }
}
