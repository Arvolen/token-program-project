use anchor_lang::prelude::*;
use anchor_spl::token;
use anchor_spl::token::{Token, MintTo, Transfer, Burn, TokenAccount, Approve};

declare_id!("43uxjSHdLq7ahspsXwkCBLBh3qmgLf6A38RBbJgHycfK");

#[program]
pub mod my_token_program {
    use super::*;

    pub fn mint_token(ctx: Context<MintToken>, amount: u64) -> Result<()> {
        // Log minting details
        msg!("Minting {} tokens to account: {}", amount, ctx.accounts.token_account.key());
        msg!("Mint authority: {}", ctx.accounts.authority.key());
        // Create the MintTo struct for our context
        let cpi_accounts = MintTo {
            mint: ctx.accounts.mint.to_account_info(),
            to: ctx.accounts.token_account.to_account_info(),
            authority: ctx.accounts.authority.to_account_info(),
        };
        
        let cpi_program = ctx.accounts.token_program.to_account_info();
        // Create the CpiContext we need for the request
        let cpi_ctx = CpiContext::new(cpi_program, cpi_accounts);

        // Execute anchor's helper function to mint tokens
        token::mint_to(cpi_ctx, amount)?;
        
        Ok(())
    }

    pub fn transfer_token(ctx: Context<TransferToken>, amount: u64) -> Result<()> {
        // Log transfer details
        msg!("Transferring {} tokens from account: {} to account: {}", amount, ctx.accounts.from.key(), ctx.accounts.to.key());
        msg!("Transfer authority: {}", ctx.accounts.from_authority.key());
        // Create the Transfer struct for our context
        let transfer_instruction = Transfer{
            from: ctx.accounts.from.to_account_info(),
            to: ctx.accounts.to.to_account_info(),
            authority: ctx.accounts.from_authority.to_account_info(),
        };
         
        let cpi_program = ctx.accounts.token_program.to_account_info();
        // Create the Context for our Transfer request
        let cpi_ctx = CpiContext::new(cpi_program, transfer_instruction);

        // Execute anchor's helper function to transfer tokens
        anchor_spl::token::transfer(cpi_ctx, amount)?;
 
        Ok(())
    }

    pub fn burn_token(ctx: Context<BurnToken>, amount: u64) -> Result<()> {
        // Log burn details
        msg!("Burning {} tokens from account: {}", amount, ctx.accounts.token_account.key());
        // Create the Burn struct for our context
        let cpi_accounts = Burn {
            mint: ctx.accounts.mint.to_account_info(),
            from: ctx.accounts.token_account.to_account_info(),
            authority: ctx.accounts.authority.to_account_info(),
        };

        let cpi_program = ctx.accounts.token_program.to_account_info();
        // Create the CpiContext we need for the request
        let cpi_ctx = CpiContext::new(cpi_program, cpi_accounts);

        // Execute anchor's helper function to burn tokens
        token::burn(cpi_ctx, amount)?;

        Ok(())
    }

    pub fn approve_delegate(ctx: Context<ApproveDelegate>, amount: u64) -> Result<()> {
        // Log approval details
        msg!("Approving {} tokens for delegate: {}", amount, ctx.accounts.delegate.key());
        // Create the Approve struct for our context
        let cpi_accounts = Approve {
            to: ctx.accounts.token_account.to_account_info(),
            delegate: ctx.accounts.delegate.to_account_info(),
            authority: ctx.accounts.authority.to_account_info(),
        };

        let cpi_program = ctx.accounts.token_program.to_account_info();
        // Create the CpiContext we need for the request
        let cpi_ctx = CpiContext::new(cpi_program, cpi_accounts);

        // Execute anchor's helper function to approve delegate
        token::approve(cpi_ctx, amount)?;

        Ok(())
    }

    pub fn get_balance(ctx: Context<GetBalance>) -> Result<u64> {
        let account = &ctx.accounts.token_account;
        let balance = account.amount;
        msg!("Account balance: {} tokens", balance);

        Ok(balance)
    }
}

#[derive(Accounts)]
pub struct MintToken<'info> {
    /// CHECK: This is the token that we want to mint
    #[account(mut)]
    pub mint: UncheckedAccount<'info>,
    pub token_program: Program<'info, Token>,
    /// CHECK: This is the token account that we want to mint tokens to
    #[account(mut)]
    pub token_account: UncheckedAccount<'info>,
    /// CHECK: the authority of the mint account
    #[account(mut)]
    pub authority: AccountInfo<'info>,
}

#[derive(Accounts)]
pub struct TransferToken<'info> {
    pub token_program: Program<'info, Token>,
    /// CHECK: The associated token account that we are transferring the token from
    #[account(mut)]
    pub from: UncheckedAccount<'info>,
    /// CHECK: The associated token account that we are transferring the token to
    #[account(mut)]
    pub to: AccountInfo<'info>,
    // the authority of the from account 
    pub from_authority: Signer<'info>,
}

#[derive(Accounts)]
pub struct BurnToken<'info> {
    /// CHECK: This is the token that we want to burn
    #[account(mut)]
    pub mint: UncheckedAccount<'info>,
    pub token_program: Program<'info, Token>,
    /// CHECK: This is the token account that we want to burn tokens from
    #[account(mut)]
    pub token_account: UncheckedAccount<'info>,
    /// CHECK: the authority of the burn account
    #[account(mut)]
    pub authority: AccountInfo<'info>,
}

#[derive(Accounts)]
pub struct ApproveDelegate<'info> {
    pub token_program: Program<'info, Token>,
    /// CHECK: The associated token account
    #[account(mut)]
    pub token_account: UncheckedAccount<'info>,
    /// CHECK: The delegate account
    #[account(mut)]
    pub delegate: UncheckedAccount<'info>,
    // the authority of the token account
    pub authority: Signer<'info>,
}

#[derive(Accounts)]
pub struct GetBalance<'info> {
    /// CHECK: The token account we want to check the balance of
    #[account(mut)]
    pub token_account: Account<'info, TokenAccount>,
}
