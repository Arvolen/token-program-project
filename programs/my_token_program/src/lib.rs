use anchor_lang::prelude::*;
use anchor_spl::token;
use anchor_spl::token::{Token, MintTo, Transfer, Burn, TokenAccount, Approve};

use {
    anchor_spl::
        token_2022::spl_token_2022::{
                extension::{
                    transfer_hook::{TransferHookAccount},
                    BaseStateWithExtensions, StateWithExtensions,
                },
                state::Account as Token2022Account,
            },
    spl_transfer_hook_interface::error::TransferHookError,
};

use anchor_spl::{
    associated_token::AssociatedToken,
    token::{Mint},
    metadata::{
        create_metadata_accounts_v3,
        mpl_token_metadata::types::DataV2,
        CreateMetadataAccountsV3, 
        Metadata as Metaplex,
    },
};

declare_id!("2GzYGFTqCsg4StJsj6REvL5PL3hLje276o5bBknCBoxR");

// Sha256(spl-transfer-hook-interface:execute)[..8]
pub const EXECUTE_IX_TAG_LE: [u8; 8] = [105, 37, 101, 197, 75, 251, 102, 26];

fn check_token_account_is_transferring(account_data: &[u8]) -> Result<()> {
	let token_account = StateWithExtensions::<Token2022Account>::unpack(account_data)?;
	let extension = token_account.get_extension::<TransferHookAccount>()?;
	if bool::from(extension.transferring) {
		Ok(())
	} else {
		Err(Into::<ProgramError>::into(
			TransferHookError::ProgramCalledOutsideOfTransfer,
		))?
	}
}



#[program]
pub mod my_token_program {

    use solana_program::program::invoke_signed;
    use solana_program::system_instruction;
    use spl_transfer_hook_interface::collect_extra_account_metas_signer_seeds;
    use spl_transfer_hook_interface::instruction::ExecuteInstruction;
    use spl_tlv_account_resolution::state::ExtraAccountMetaList;
    use spl_tlv_account_resolution::account::ExtraAccountMeta;
    use spl_pod::primitives::PodBool;


    use super::*;

    pub fn initialize(ctx: Context<Initialize>) -> Result<()> {
        let counter = &mut ctx.accounts.counter;
        counter.owner = *ctx.accounts.authority.key;
        counter.count = 0;
        Ok(())
    }


    pub fn init_token(ctx: Context<InitToken>, metadata: InitTokenParams) -> Result<()> {
        let seeds = &["mint".as_bytes(), &[ctx.bumps.mint]];
        let signer = [&seeds[..]];

        let token_data: DataV2 = DataV2 {
            name: metadata.name,
            symbol: metadata.symbol,
            uri: metadata.uri,
            seller_fee_basis_points: 0,
            creators: None,
            collection: None,
            uses: None,
        };

        let metadata_ctx = CpiContext::new_with_signer(
            ctx.accounts.token_metadata_program.to_account_info(),
            CreateMetadataAccountsV3 {
                payer: ctx.accounts.payer.to_account_info(),
                update_authority: ctx.accounts.mint.to_account_info(),
                mint: ctx.accounts.mint.to_account_info(),
                metadata: ctx.accounts.metadata.to_account_info(),
                mint_authority: ctx.accounts.mint.to_account_info(),
                system_program: ctx.accounts.system_program.to_account_info(),
                rent: ctx.accounts.rent.to_account_info(),
            },
            &signer
        );

        create_metadata_accounts_v3(
            metadata_ctx,
            token_data,
            false,
            true,
            None,
        )?;

        msg!("Token mint created successfully.");

        Ok(())
    }

    pub fn mint_token(ctx: Context<MintToken>,) -> Result<()> {
        // Log minting details
        msg!("Minting 10 tokens to account: {}", ctx.accounts.token_account.key());
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
        token::mint_to(cpi_ctx, 10)?;
        
        Ok(())
    }

    pub fn transfer_token(ctx: Context<TransferToken>) -> Result<()> {
       // Log transfer details
       msg!("Transferring 5 tokens from account: {} to account: {}", ctx.accounts.from.key(), ctx.accounts.to.key());
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
        anchor_spl::token::transfer(cpi_ctx, 5)?;
 
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

    pub fn transfer_hook<'a>(ctx: Context<TransferHook>, amount: u64) -> Result<()> {
        // Count the number of times the transfer hook has been invoked.
        let counter = &mut ctx.accounts.counter;
        counter.count += 1;

        let source_account = &ctx.accounts.source;
    	let destination_account = &ctx.accounts.destination;

        check_token_account_is_transferring(&source_account.to_account_info().try_borrow_data()?)?;
    	check_token_account_is_transferring(&destination_account.to_account_info().try_borrow_data()?)?;

        msg!("Transfer hook invoked");
        msg!("Transfer amount: {}", amount);
        msg!("Transfer with extra account PDA: {}", ctx.accounts.extra_account.key());
        msg!("Transfer with counter.count: {}", counter.count);
        Ok(())
    }

    pub fn initialize_extra_account_meta_list(ctx: Context<InitializeExtraAccountMetaList>, bump_seed: u8) -> Result<()> {
        // Create the extra account meta list.
        let account_metas = vec![
            ExtraAccountMeta {
                discriminator: 0,
                address_config: ctx.accounts.counter.key().to_bytes(),
                is_signer: PodBool::from(false),
                is_writable: PodBool::from(true),
            }];

        // Allocate extra account PDA account.
        let bump_seed = [bump_seed];
        let signer_seeds = collect_extra_account_metas_signer_seeds(ctx.accounts.mint.key, &bump_seed);
        let account_size = ExtraAccountMetaList::size_of(account_metas.len())?;
        invoke_signed(
            &system_instruction::allocate(ctx.accounts.extra_account.key, account_size as u64),
            &[ctx.accounts.extra_account.clone()],
            &[&signer_seeds],
        )?;
        invoke_signed(
            &system_instruction::assign(ctx.accounts.extra_account.key, ctx.program_id),
            &[ctx.accounts.extra_account.clone()],
            &[&signer_seeds],
        )?;

        // Write the extra account meta list to the extra account PDA.
        let mut data = ctx.accounts.extra_account.try_borrow_mut_data()?;
        ExtraAccountMetaList::init::<ExecuteInstruction>(&mut data, &account_metas)?;

        msg!("Extra account meta list initialized on {}", ctx.accounts.extra_account.key());
        Ok(())
    }

    pub fn fallback<'a>(program_id: &Pubkey, accounts: &'a[AccountInfo<'a>], ix_data: &[u8]) -> Result<()> {
        let mut ix_data: &[u8] = ix_data;
        let sighash: [u8; 8] = {
            let mut sighash: [u8; 8] = [0; 8];
            sighash.copy_from_slice(&ix_data[..8]);
            ix_data = &ix_data[8..];
            sighash
        };
        match sighash {
            EXECUTE_IX_TAG_LE => {__private::__global::transfer_hook(program_id, accounts, ix_data)},
            _ => Err(ProgramError::InvalidInstructionData.into()),
        }
    }
}

#[derive(Accounts)]
pub struct Initialize<'info> {
    #[account(init, 
        seeds = [authority.key().as_ref()], 
        bump, 
        payer = authority, 
        space = 8 + 128)
    ]
    pub counter: Account<'info, Counter>,
    pub system_program: Program<'info, System>,
    #[account(mut)]
    pub authority: Signer<'info>,
}

#[derive(Accounts)]
#[instruction(
    params: InitTokenParams
)]
pub struct InitToken<'info> {
    /// CHECK: New Metaplex Account being created
    #[account(mut)]
    pub metadata: UncheckedAccount<'info>,
    #[account(
        init,
        seeds = [b"mint"],
        bump,
        payer = payer,
        mint::decimals = params.decimals,
        mint::authority = mint,
    )]
    pub mint: Account<'info, Mint>,
    #[account(mut)]
    pub payer: Signer<'info>,
    pub rent: Sysvar<'info, Rent>,
    pub system_program: Program<'info, System>,
    pub token_program: Program<'info, Token>,
    pub token_metadata_program: Program<'info, Metaplex>,
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

#[derive(Accounts)]
#[instruction(amount: u64)]
pub struct TransferHook<'info> {
    /// CHECK:
    pub source: AccountInfo<'info>,
    /// CHECK:
    pub mint: AccountInfo<'info>,
    /// CHECK:
    pub destination: AccountInfo<'info>,
    /// CHECK:
    pub authority: AccountInfo<'info>,
    /// CHECK: must be the extra account PDA
    #[account(
        seeds = [b"extra-account-metas", mint.key().as_ref()], 
        bump)
    ]
    pub extra_account: AccountInfo<'info>,
    /// CHECK:
    pub counter: Account<'info, Counter>,
}

#[derive(Accounts)]
pub struct InitializeExtraAccountMetaList<'info> {
    /// CHECK: must be the extra account PDA
    #[account(mut,
        seeds = [b"extra-account-metas", mint.key().as_ref()], 
        bump)
    ]
    pub extra_account: AccountInfo<'info>,
    #[account(mut)]
    pub counter: Account<'info, Counter>,
    /// CHECK:
    pub mint: AccountInfo<'info>,
    /// CHECK:
    pub authority: AccountInfo<'info>,
    pub system_program: Program<'info, System>,
}

#[account]
pub struct Counter {
    pub owner: Pubkey,
    pub count: u64,
}

#[derive(AnchorSerialize, AnchorDeserialize, Debug, Clone)]
pub struct InitTokenParams {
    pub name: String,
    pub symbol: String,
    pub uri: String,
    pub decimals: u8,
}