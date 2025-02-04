use anchor_lang::prelude::*;
use anchor_spl::{
    associated_token::AssociatedToken,
    token::{mint_to, transfer, Mint, MintTo, Token, TokenAccount, Transfer},
};
use constant_product_curve::ConstantProduct;

use crate::error::*;
use crate::state::*;

#[derive(Accounts)]
pub struct Deposit<'info> {
    #[account(mut)]
    pub lp_provider: Signer<'info>,
    pub mint_x: Account<'info, Mint>,
    pub mint_y: Account<'info, Mint>,
    #[account(
      has_one = mint_x,
      has_one = mint_y,
      seeds = [b"config", config.seed.to_le_bytes().as_ref()], 
      bump = config.config_bump
    )]
    pub config: Account<'info, Config>,
    #[account(
        mint::decimals = 6,
        mint::authority = config,
        seeds = [b"lp", config.key().as_ref()], 
        bump = config.lp_bump
    )]
    pub mint_lp: Account<'info, Mint>,
    #[account(
        mut,
        associated_token::mint = mint_x,
        associated_token::authority = config,
    )]
    pub vault_x: Account<'info, TokenAccount>,
    #[account(
        mut,
        associated_token::mint = mint_y,
        associated_token::authority = config,
    )]
    pub vault_y: Account<'info, TokenAccount>,
    #[account(
      init_if_needed,
      payer = lp_provider,
      associated_token::mint = mint_x,
      associated_token::authority = lp_provider,
    )]
    pub lp_provider_mint_x: Account<'info, TokenAccount>,
    #[account(
      init_if_needed,
      payer = lp_provider,
      associated_token::mint = mint_y,
      associated_token::authority = lp_provider,
    )]
    pub lp_provider_mint_y: Account<'info, TokenAccount>,
    #[account(
      init_if_needed,
      payer = lp_provider,
      associated_token::mint = mint_lp,
      associated_token::authority = lp_provider,
    )]
    pub lp_provider_mint_lp: Account<'info, TokenAccount>,
    pub system_program: Program<'info, System>,
    pub token_program: Program<'info, Token>,
    pub associated_token_program: Program<'info, AssociatedToken>,
}

impl<'info> Deposit<'info> {
    pub fn deposit(&mut self, amount: u64, max_x: u64, max_y: u64) -> Result<()> {
        require!(self.config.locked == false, AmmError::PoolLocked);
        require!(amount != 0, AmmError::InvalidAmount);

        let (x, y) = match self.mint_lp.supply == 0
            && self.vault_x.amount == 0
            && self.vault_y.amount == 0
        {
            true => (max_x, max_y),
            false => {
                let amounts = ConstantProduct::xy_deposit_amounts_from_l(
                    self.vault_x.amount,
                    self.vault_y.amount,
                    self.mint_lp.supply,
                    amount,
                    6,
                )
                .unwrap();
                (amounts.x, amounts.y)
            }
        };

        require!(x <= max_x && y <= max_y, AmmError::SlippageExceeded);

        // deposit token x
        self.deposit_token(true, x)?;
        // deposit token y
        self.deposit_token(false, y)?;
        // mint lp token
        self.mint_lp_token(amount)?;
        Ok(())
    }

    pub fn deposit_token(&mut self, is_x: bool, amount: u64) -> Result<()> {
        let (from, to) = match is_x {
            true => (
                self.lp_provider_mint_x.to_account_info(),
                self.vault_x.to_account_info(),
            ),
            false => (
                self.lp_provider_mint_y.to_account_info(),
                self.vault_y.to_account_info(),
            ),
        };
        let program = self.token_program.to_account_info();
        let account = Transfer {
            from,
            to,
            authority: self.lp_provider.to_account_info(),
        };

        let ctx = CpiContext::new(program, account);

        transfer(ctx, amount)?;
        Ok(())
    }

    pub fn mint_lp_token(&mut self, amount: u64) -> Result<()> {
        let program = self.token_program.to_account_info();
        let account = MintTo {
            mint: self.mint_lp.to_account_info(),
            to: self.lp_provider_mint_lp.to_account_info(),
            authority: self.config.to_account_info(),
        };

        let seeds = &[
            &b"config"[..],
            &self.config.seed.to_le_bytes(),
            &[self.config.config_bump],
        ];

        let signer_seeds = &[&seeds[..]];

        let ctx = CpiContext::new_with_signer(program, account, signer_seeds);

        mint_to(ctx, amount)?;
        Ok(())
    }
}
