use anchor_lang::prelude::*;
use anchor_spl::{
    associated_token::AssociatedToken,
    token::{close_account, transfer_checked, CloseAccount, TransferChecked},
    token_interface::{Mint, TokenAccount, TokenInterface},
};

use crate::states::*;

#[derive(Accounts)]
#[instruction()]
pub struct Take<'info> {
    #[account(mut)]
    pub taker: Signer<'info>,
    #[account(mut)]
    pub maker: SystemAccount<'info>,
    pub mint_a: InterfaceAccount<'info, Mint>,
    pub mint_b: InterfaceAccount<'info, Mint>,
    #[account(
      init_if_needed,
      payer=taker,
      associated_token::mint=mint_a,
      associated_token::authority=taker,
  )]
    pub taker_mint_a_ata: Box<InterfaceAccount<'info, TokenAccount>>,
    #[account(
      mut,
      associated_token::mint=mint_b,
      associated_token::authority=taker,
  )]
    pub taker_mint_b_ata: Box<InterfaceAccount<'info, TokenAccount>>,
    #[account(
      init_if_needed,
      payer=taker,
      associated_token::mint=mint_b,
      associated_token::authority=maker,
  )]
    pub maker_mint_b_ata: Box<InterfaceAccount<'info, TokenAccount>>,
    #[account(
      mut,
      close=taker,
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

impl<'info> Take<'info> {
    pub fn withdraw(&mut self) -> Result<()> {
        let cpi_program = self.token_program.to_account_info();

        let cpi_account = TransferChecked {
            from: self.taker_mint_b_ata.to_account_info(),
            mint: self.mint_b.to_account_info(),
            to: self.maker_mint_b_ata.to_account_info(),
            authority: self.taker.to_account_info(),
        };
        let cpi_ctx = CpiContext::new(cpi_program.clone(), cpi_account);

        // Transfer token b from taker to maker
        transfer_checked(cpi_ctx, self.escrow.receive_amount, self.mint_b.decimals)?;

        let cpi_account = TransferChecked {
            from: self.vault.to_account_info(),
            mint: self.mint_a.to_account_info(),
            to: self.taker_mint_a_ata.to_account_info(),
            authority: self.escrow.to_account_info(),
        };

        let maker_binding = self.escrow.maker.to_bytes();
        let seed_binding = self.escrow.seed.to_le_bytes();
        let bump_binding = self.escrow.bump;

        let seeds: [&[u8]; 4] = [b"escrow", &maker_binding, &seed_binding, &[bump_binding]];
        let signer_seeds: &[&[&[u8]]] = &[&seeds];
        let cpi_ctx = CpiContext::new_with_signer(cpi_program, cpi_account, &signer_seeds);

        // Transfer token a from vault to taker
        transfer_checked(cpi_ctx, self.vault.amount, self.mint_a.decimals)?;

        Ok(())
    }

    pub fn close(&mut self) -> Result<()> {
        let cpi_program = self.token_program.to_account_info();

        let cpi_account = CloseAccount {
            authority: self.escrow.to_account_info(),
            account: self.vault.to_account_info(),
            destination: self.taker.to_account_info(),
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
