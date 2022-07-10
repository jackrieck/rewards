use anchor_lang::prelude::*;
use anchor_lang::solana_program::{
    program::invoke_signed,
    sysvar::instructions::{load_current_index_checked, load_instruction_at_checked},
};
use anchor_spl::{self, associated_token, token};
use mpl_token_metadata::instruction::create_metadata_accounts_v2;
use mpl_token_metadata::state::{DataV2, UseMethod, Uses};

declare_id!("Fg6PaFpoGXkYsidMpWTK6W2BeZ7FEfcYkg476zPFsLnS");

#[program]
pub mod rewards {

    use super::*;

    // admin creates a reward plan for their service
    pub fn create_reward_plan(
        ctx: Context<CreateRewardPlan>,
        params: CreateRewardPlanParams,
    ) -> Result<()> {
        // create fungible metadata account
        // create collection metadata account
        let data = DataV2 {
            name: params.name.clone(),
            symbol: params.metadata_symbol,
            uri: params.metadata_uri.clone(),
            seller_fee_basis_points: 0,
            creators: None,
            collection: None,
            uses: Some(Uses {
                use_method: UseMethod::Multiple,
                remaining: 100_000_000,
                total: 100_000_000,
            }),
        };

        let create_metadata_accounts = [
            ctx.accounts.metadata.to_account_info(),
            ctx.accounts.mint.to_account_info(),
            ctx.accounts.manager.to_account_info(),
            ctx.accounts.admin.to_account_info(),
            ctx.accounts.manager.to_account_info(),
            ctx.accounts.system_program.to_account_info(),
            ctx.accounts.rent.to_account_info(),
        ];

        // TODO: once we can bump latest version use v3
        let metadata_ix = create_metadata_accounts_v2(
            ctx.accounts.metadata_program.key(),
            ctx.accounts.metadata.key(),
            ctx.accounts.mint.to_account_info().key(),
            ctx.accounts.manager.key(),
            ctx.accounts.admin.key(),
            ctx.accounts.manager.key(),
            data.name,
            data.symbol,
            data.uri,
            data.creators,
            data.seller_fee_basis_points,
            false,
            false,
            data.collection,
            data.uses,
        );

        invoke_signed(
            &metadata_ix,
            &create_metadata_accounts,
            &[&[
                ctx.accounts.admin.key().as_ref(),
                params.name.as_bytes(),
                &[*ctx.bumps.get("manager").unwrap()],
            ]],
        )?;

        // set manager in account
        let manager = &mut ctx.accounts.manager;
        manager.name = params.name;
        manager.threshold = params.threshold;
        manager.allowed_program = params.allowed_program;

        Ok(())
    }

    pub fn end_reward_plan(_ctx: Context<EndRewardPlan>, _name: String) -> Result<()> {
        Ok(())
    }

    pub fn reward(ctx: Context<Reward>, params: RewardParams) -> Result<bool> {
        // get calling program_id
        let current_index = load_current_index_checked(&ctx.accounts.instructions)? as usize;
        let current_ix = load_instruction_at_checked(current_index, &ctx.accounts.instructions)?;

        // if the caller is not the managerured allowed program return an error
        if current_ix
            .program_id
            .ne(&ctx.accounts.manager.allowed_program)
        {
            return err!(ErrorCodes::InsufficientPrivileges);
        }

        // check if approved before minting the next reward token
        let is_approved = ctx.accounts.manager.threshold <= ctx.accounts.user_ata.amount;

        if is_approved {
            // if user has reached required amount of tokens, burn the amount so they start over fresh
            let burn_accounts = token::Burn {
                mint: ctx.accounts.mint.to_account_info(),
                from: ctx.accounts.user_ata.to_account_info(),
                authority: ctx.accounts.user.to_account_info(),
            };

            token::burn(
                CpiContext::new(ctx.accounts.token_program.to_account_info(), burn_accounts),
                ctx.accounts.manager.threshold,
            )?;
        }

        // mint new reward token
        let mint_to_accounts = token::MintTo {
            mint: ctx.accounts.mint.to_account_info(),
            to: ctx.accounts.user_ata.to_account_info(),
            authority: ctx.accounts.manager.to_account_info(),
        };

        token::mint_to(
            CpiContext::new_with_signer(
                ctx.accounts.token_program.to_account_info(),
                mint_to_accounts,
                &[&[
                    params.admin.as_ref(),
                    params.name.as_bytes(),
                    &[*ctx.bumps.get("manager").unwrap()],
                ]],
            ),
            params.amount,
        )?;

        Ok(is_approved)
    }
}

#[derive(Accounts)]
#[instruction(params: CreateRewardPlanParams)]
pub struct CreateRewardPlan<'info> {
    #[account(init, payer = admin, seeds = [manager.key().as_ref()], bump, mint::decimals = 6, mint::authority = manager)]
    pub mint: Account<'info, token::Mint>,

    /// CHECK: todo
    #[account(mut, seeds = [b"metadata", mpl_token_metadata::ID.as_ref(), mint.key().as_ref()], bump, seeds::program = mpl_token_metadata::ID)]
    pub metadata: AccountInfo<'info>,

    #[account(init, space = RewardPlanManager::MAX_SIZE, payer = admin, seeds = [admin.key().as_ref(), params.name.as_bytes()], bump)]
    pub manager: Account<'info, RewardPlanManager>,

    #[account(mut)]
    pub admin: Signer<'info>,

    pub system_program: Program<'info, System>,
    pub rent: Sysvar<'info, Rent>,
    pub token_program: Program<'info, token::Token>,
    pub associated_token_program: Program<'info, associated_token::AssociatedToken>,
    pub metadata_program: Program<'info, TokenMetadata>,
}

#[account]
#[derive(Default)]
pub struct RewardPlanManager {
    pub name: String,
    pub threshold: u64,
    pub allowed_program: Pubkey,
}

impl RewardPlanManager {
    pub const MAX_SIZE: usize = 8 + 50 + 8 + 32;
}

#[derive(AnchorSerialize, AnchorDeserialize)]
pub struct CreateRewardPlanParams {
    pub name: String,
    pub threshold: u64,
    pub allowed_program: Pubkey,
    pub metadata_uri: String,
    pub metadata_symbol: String,
}

#[derive(Accounts)]
pub struct EndRewardPlan<'info> {
    #[account(mut)]
    pub user: Signer<'info>,
}

#[derive(Accounts)]
#[instruction(params: RewardParams)]
pub struct Reward<'info> {
    #[account(mut, seeds = [manager.key().as_ref()], bump, mint::decimals = 6, mint::authority = manager)]
    pub mint: Account<'info, token::Mint>,

    #[account(seeds = [params.admin.as_ref(), params.name.as_bytes()], bump)]
    pub manager: Account<'info, RewardPlanManager>,

    #[account(mut)]
    pub user: Signer<'info>,

    #[account(init_if_needed, payer = user, associated_token::mint = mint, associated_token::authority = user)]
    pub user_ata: Account<'info, token::TokenAccount>,

    /// CHECK: todo
    #[account(address = anchor_lang::solana_program::sysvar::instructions::ID)]
    pub instructions: AccountInfo<'info>,

    pub system_program: Program<'info, System>,
    pub rent: Sysvar<'info, Rent>,
    pub token_program: Program<'info, token::Token>,
    pub associated_token_program: Program<'info, associated_token::AssociatedToken>,
}

#[derive(AnchorSerialize, AnchorDeserialize)]
pub struct RewardParams {
    pub name: String,
    pub amount: u64,
    pub admin: Pubkey,
}

// Create an anchor compatible mpl_token_metadata struct
#[derive(Clone)]
pub struct TokenMetadata;

impl anchor_lang::Id for TokenMetadata {
    fn id() -> Pubkey {
        mpl_token_metadata::ID
    }
}

#[error_code]
pub enum ErrorCodes {
    #[msg("program doesn't have sufficient privileges to call Reward")]
    InsufficientPrivileges,
}
