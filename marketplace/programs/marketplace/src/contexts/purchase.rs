use anchor_lang::{prelude::*, system_program::{transfer, Transfer}};
use anchor_spl::{associated_token::AssociatedToken, token::{close_account, transfer_checked, CloseAccount, TransferChecked}, token_interface::{Mint, TokenAccount, TokenInterface}};

use crate::states::*;

#[derive(Accounts)]
#[instruction(name: String)]
pub struct Purchase<'info> {
    #[account(mut)]
    pub taker: Signer<'info>,
    #[account(mut)]
    pub maker: SystemAccount<'info>,
    pub maker_mint: InterfaceAccount<'info, Mint>,
    #[account(
      mut,
      seeds = [b"marketplace", name.as_str().as_bytes()],
      bump = marketplace.bump,
  )]
    pub marketplace: Account<'info, Marketplace>,
    #[account(
      init_if_needed,
      payer = taker,
      associated_token::mint = maker_mint,
      associated_token::authority = taker,
    )]
    pub taker_ata: InterfaceAccount<'info, TokenAccount>,
    #[account(
        mut,
        associated_token::mint = maker_mint,
        associated_token::authority = listing,
    )]
    pub vault: InterfaceAccount<'info, TokenAccount>,
    #[account(
        mut,
        close = maker,
        seeds = [marketplace.key().as_ref(), maker_mint.key().as_ref()], 
        bump = listing.bump
    )]
    pub listing: Account<'info, Listing>,
    #[account(
        mut,
        seeds = [b"treasury", marketplace.key().as_ref()], 
        bump = marketplace.treasury_bump
    )]
    pub treasury: SystemAccount<'info>,
    #[account(
        mut,
        seeds = [b"rewards", marketplace.key().as_ref()],
        mint::authority = marketplace,
        mint::decimals = 6,
        bump = marketplace.rewards_bump
    )]
    pub rewards_mint: InterfaceAccount<'info, Mint>,
    pub token_program: Interface<'info, TokenInterface>,
    pub system_program: Program<'info, System>,
    pub associated_token_program: Program<'info, AssociatedToken>,
}

impl<'info> Purchase<'info> {
  // send sol and take fee
  pub fn purchase(&mut self) -> Result<()> {
    let program = self.system_program.to_account_info();
    let accounts = Transfer {
      from: self.taker.to_account_info(),
      to: self.maker.to_account_info(),
    };

    let ctx = CpiContext::new(program, accounts);

    let amount = self.listing.price
        .checked_sub(self.marketplace.fee as u64).unwrap();

    transfer(ctx, amount)?;

    // send fee to treasury
    let program = self.system_program.to_account_info();
    let accounts = Transfer {
      from: self.taker.to_account_info(),
      to: self.treasury.to_account_info(),
    };

    let ctx = CpiContext::new(program, accounts);

    transfer(ctx, self.marketplace.fee as u64)?;

    Ok(())
  }

  // transfer NFT to buyer
  pub fn transfer_nft(&mut self) -> Result<()> {
    let program = self.token_program.to_account_info();
    let account = TransferChecked {
        from: self.vault.to_account_info(),
        mint: self.maker_mint.to_account_info(),
        to: self.taker_ata.to_account_info(),
        authority: self.listing.to_account_info(),
    };
    let seeds = &[
        self.marketplace.to_account_info().key.as_ref(),
        self.maker_mint.to_account_info().key.as_ref(),
        &[self.listing.bump],
    ];
    let signer_seeds: &[&[&[u8]]] = &[&seeds[..]];

    let ctx = CpiContext::new_with_signer(program, account, signer_seeds);

    transfer_checked(ctx, 1, self.maker_mint.decimals)?;

    Ok(())
}

  // clean listing account
  pub fn close(&mut self) -> Result<()> {
    let program = self.token_program.to_account_info();

    let account = CloseAccount {
        authority: self.listing.to_account_info(),
        account: self.vault.to_account_info(),
        destination: self.maker.to_account_info(),
    };
    let seeds = &[
        self.marketplace.to_account_info().key.as_ref(),
        self.maker_mint.to_account_info().key.as_ref(),
        &[self.listing.bump],
    ];
    let signer_seeds: &[&[&[u8]]] = &[&seeds[..]];
    let ctx = CpiContext::new_with_signer(program, account, &signer_seeds);

    close_account(ctx)?;

    Ok(())
  }
}
