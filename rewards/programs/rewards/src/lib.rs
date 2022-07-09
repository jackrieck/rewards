use anchor_lang::prelude::*;
use anchor_lang::solana_program::program::invoke;
use anchor_spl::{self, associated_token, token};
use mpl_token_metadata::instruction::create_metadata_accounts_v2;
use mpl_token_metadata::state::{Uses, UseMethod, DataV2};

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
            ctx.accounts.admin.to_account_info(),
            ctx.accounts.admin.to_account_info(),
            ctx.accounts.admin.to_account_info(),
            ctx.accounts.system_program.to_account_info(),
            ctx.accounts.rent.to_account_info(),
        ];

        // TODO: once we can bump latest version use v3
        let metadata_ix = create_metadata_accounts_v2(
            ctx.accounts.metadata_program.key(),
            ctx.accounts.metadata.key(),
            ctx.accounts.mint.to_account_info().key(),
            ctx.accounts.admin.key(),
            ctx.accounts.admin.key(),
            ctx.accounts.admin.key(),
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

        invoke(
            &metadata_ix,
            &create_metadata_accounts,
        )?; 

        Ok(())
    }

    pub fn end_reward_plan(_ctx: Context<EndRewardPlan>, _name: String) -> Result<()> {
        Ok(())
    }

    pub fn apply(_ctx: Context<Approve>) -> Result<bool> {
        Ok(true)
    }
}

#[derive(Accounts)]
#[instruction(params: CreateRewardPlanParams)]
pub struct CreateRewardPlan<'info> {
    #[account(init, payer = admin, seeds = [config.key().as_ref()], bump, mint::decimals = 6, mint::authority = admin)]
    pub mint: Account<'info, token::Mint>,

    /// CHECK: todo
    #[account(mut, seeds = [b"metadata", mpl_token_metadata::ID.as_ref(), mint.key().as_ref()], bump, seeds::program = mpl_token_metadata::ID)]
    pub metadata: AccountInfo<'info>,

    #[account(init, space = RewardPlanConfig::MAX_SIZE, payer = admin, seeds = [admin.key().as_ref(), params.name.as_bytes()], bump)]
    pub config: Account<'info, RewardPlanConfig>,

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
pub struct RewardPlanConfig {
    pub name: String,
    pub threshold: u64,
    pub metadata_uri: String,
}

impl RewardPlanConfig {
    pub const MAX_SIZE: usize = 8 + 50 + 8 + 200 + 10;
}

#[derive(AnchorSerialize, AnchorDeserialize)]
pub struct CreateRewardPlanParams {
    pub name: String,
    pub threshold: u64,
    pub metadata_uri: String,
    pub metadata_symbol: String,
}

#[derive(Accounts)]
pub struct EndRewardPlan {}

#[derive(Accounts)]
pub struct Approve {}

// Create an anchor compatible mpl_token_metadata struct
#[derive(Clone)]
pub struct TokenMetadata;

impl anchor_lang::Id for TokenMetadata {
    fn id() -> Pubkey {
        mpl_token_metadata::ID
    }
}
