use anchor_lang::prelude::*;
use anchor_spl::token::{Token, Mint, MintTo, mint_to, TokenAccount, transfer, Transfer};
use anchor_spl::associated_token::{AssociatedToken};

declare_id!("Fg6PaFpoGXkYsidMpWTK6W2BeZ7FEfcYkg476zPFsLnS");

#[program]
pub mod vault {
    use super::*;
    pub fn initialize(ctx: Context<Initialize>) -> Result<()> {
        let vault_admin = &mut ctx.accounts.vault_admin;
        vault_admin.admin = ctx.accounts.authority.key();
        vault_admin.created_at = get_time();
        Ok(())
    }

    pub fn initialize_mint(ctx: Context<InitializeMint>) -> Result<()> {
        let key = ctx.accounts.authority.key;
        let vault_admin = &mut ctx.accounts.vault_admin;
        let current_time = get_time();
        require!(current_time >= vault_admin.created_at, VaultErrors::VaultNotInitialized);
        require!(*key == vault_admin.admin, VaultErrors::NotAdmin);
        vault_admin.mint_created_at = current_time;
        Ok(())
    }
    
    pub fn initialize_account(ctx: Context<InitializeAccount>) -> Result<()> {
        let vault_admin = &ctx.accounts.vault_admin;
        let current_time = get_time();
        let key = ctx.accounts.authority.key;
        require!(current_time >= vault_admin.created_at, VaultErrors::VaultNotInitialized);
        require!(current_time >= vault_admin.mint_created_at, VaultErrors::MintNotInitialized);
        require!(*key == vault_admin.admin, VaultErrors::NotAdmin);
        Ok(())
    }

    pub fn mint_token(ctx: Context<MintToken>, amount: u64) -> Result<()> {
        let vault_admin = &ctx.accounts.vault_admin;
        let key = ctx.accounts.authority.key;
        let current_time = get_time();
        require!(*key == vault_admin.admin, VaultErrors::NotAdmin);
        require!(current_time >= vault_admin.created_at, VaultErrors::VaultNotInitialized);
        require!(current_time >= vault_admin.mint_created_at, VaultErrors::MintNotInitialized);
        mint_to(ctx.accounts.build_context(), amount)
    }

    pub fn deposit_tokens(ctx: Context<DepositTokens>, amount: u64) -> Result<()> {
        Ok(())
    }

}

fn get_time() -> i64 {
    Clock::get().unwrap().unix_timestamp
}

#[derive(Accounts)]
pub struct Initialize<'info> {
    #[account(mut)]
    pub authority: Signer<'info>,

    #[account(init, payer = authority, space = 8 + 32 + 8 + 10)]
    pub vault_admin: Account<'info, VaultAdmin>,

    pub system_program: Program<'info, System>,
}

#[account]
pub struct VaultAdmin {
    pub admin: Pubkey,
    pub created_at: i64,
    pub mint_created_at: i64,
}

#[derive(Accounts)]
pub struct InitializeMint<'info> {
    #[account(mut)]
    pub authority: Signer<'info>,
    #[account(init, payer = authority, mint::authority = authority, mint::decimals=6)]
    pub mint: Account<'info, Mint>,
    
    pub vault_admin: Account<'info, VaultAdmin>,

    pub system_program: Program<'info, System>,
    pub token_program: Program<'info, Token>,
    pub associated_token_program: Program<'info, AssociatedToken>,
    pub rent: Sysvar<'info, Rent>,
}

#[derive(Accounts)]
pub struct InitializeAccount<'info> {
    #[account(mut)]
    pub authority: Signer<'info>,

    #[account(mut)]
    pub mint: Account<'info, Mint>,

    pub vault_admin: Account<'info, VaultAdmin>,

    #[account(init, payer = authority, associated_token::mint = mint, associated_token::authority = authority)]
    pub token_account: Account<'info, TokenAccount>,

    pub system_program: Program<'info, System>,
    pub token_program: Program<'info, Token>,
    pub associated_token_program: Program<'info, AssociatedToken>,
    pub rent: Sysvar<'info, Rent>,
}

#[derive(Accounts)]
pub struct MintToken<'info> {
    #[account(mut)]
    pub authority: Signer<'info>,

    #[account(mut)]
    pub mint: Account<'info, Mint>,

    pub vault_admin: Account<'info, VaultAdmin>,

    #[account(mut)]
    pub token_account: Account<'info, TokenAccount>,

    pub token_program: Program<'info, Token>,
}

impl<'info> MintToken<'info> {
    pub fn build_context(&self) -> CpiContext<'_, '_, '_, 'info, MintTo<'info>> {
        let program = self.token_program.to_account_info();
        let accounts = MintTo{
            to: self.token_account.to_account_info(),
            mint: self.mint.to_account_info(),
            authority: self.authority.to_account_info(),
        };
        CpiContext::new(program, accounts)
    }
}

#[derive(Accounts)]
pub struct DepositTokens<'info> {
    #[account(mut)]
    pub authority: Signer<'info>,

    #[account(mut)] //TODO: attempt to remove mut
    pub mint: Account<'info, Mint>,

    #[account(mut)]
    pub user_account: Account<'info, TokenAccount>,

    #[account(mut)]
    pub vault_account: Account<'info, TokenAccount>,

    pub token_program: Program<'info, Token>,
}

impl<'info> DepositTokens<'info> {
    pub fn build_context(&self) -> CpiContext<'_, '_, '_, 'info, Transfer<'info>> {
        let program = self.token_program.to_account_info();
        let accounts = Transfer{
            authority: self.authority.to_account_info(),
            to: self.vault_account.to_account_info(),
            from: self.user_account.to_account_info(),
        };
        CpiContext::new(program, accounts)
    }
}

#[error_code]
pub enum VaultErrors {
    NotAdmin,
    VaultNotInitialized,
    MintNotInitialized,
}
