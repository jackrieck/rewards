use anchor_lang::prelude::*;
use anchor_lang::solana_program::program::invoke;
use anchor_spl::{self, associated_token, token};
use mpl_token_metadata::instruction::{create_master_edition_v3, create_metadata_accounts_v2};
use mpl_token_metadata::state::{Collection, DataV2};

declare_id!("Fg6PaFpoGXkYsidMpWTK6W2BeZ7FEfcYkg476zPFsLnS");

#[program]
pub mod rewards {
    use super::*;

    // admin creates a reward plan for their service
    // a collection NFT is minted for future use when minting NFTs used to keep track of customer activity
    // configuration is also set
    pub fn create_reward_plan(
        ctx: Context<CreateRewardPlan>,
        params: CreateRewardPlanParams,
    ) -> Result<()> {
        // mint collection token to admin
        let collection_mint_to_accounts = token::MintTo {
            mint: ctx.accounts.collection_mint.to_account_info(),
            to: ctx.accounts.collection_mint_ata.to_account_info(),
            authority: ctx.accounts.admin.to_account_info(),
        };

        token::mint_to(
            CpiContext::new(
                ctx.accounts.token_program.to_account_info(),
                collection_mint_to_accounts,
            ),
            1,
        )?;

        // create collection metadata account
        let collection_data = DataV2 {
            name: params.name.clone(),
            symbol: "REWARD".to_string(),
            uri: params.collection_metadata_uri.clone(),
            seller_fee_basis_points: 0,
            creators: None,
            collection: None,
            uses: None,
        };

        let create_collection_metadata_accounts = [
            ctx.accounts.collection_md.to_account_info(),
            ctx.accounts.collection_mint.to_account_info(),
            ctx.accounts.admin.to_account_info(),
            ctx.accounts.admin.to_account_info(),
            ctx.accounts.admin.to_account_info(),
            ctx.accounts.system_program.to_account_info(),
            ctx.accounts.rent.to_account_info(),
        ];

        // TODO: once we can bump latest version use v3
        let collection_metadata_ix = create_metadata_accounts_v2(
            ctx.accounts.metadata_program.key(),
            ctx.accounts.collection_md.key(),
            ctx.accounts.collection_mint.to_account_info().key(),
            ctx.accounts.admin.key(),
            ctx.accounts.admin.key(),
            ctx.accounts.admin.key(),
            collection_data.name,
            collection_data.symbol,
            collection_data.uri,
            collection_data.creators,
            collection_data.seller_fee_basis_points,
            false,
            false,
            collection_data.collection,
            collection_data.uses,
        );

        invoke(
            &collection_metadata_ix,
            &create_collection_metadata_accounts,
        )?;

        // create collection master edition account
        let create_collection_master_edition_accounts = [
            ctx.accounts.collection_me.to_account_info(),
            ctx.accounts.collection_md.to_account_info(),
            ctx.accounts.collection_mint.to_account_info(),
            ctx.accounts.admin.to_account_info(),
            ctx.accounts.admin.to_account_info(),
            ctx.accounts.admin.to_account_info(),
            ctx.accounts.rent.to_account_info(),
            ctx.accounts.token_program.clone().to_account_info(),
        ];

        // max_supply of 0 == unique
        let collection_master_edition_ix = create_master_edition_v3(
            ctx.accounts.metadata_program.key(),
            ctx.accounts.collection_me.key(),
            ctx.accounts.collection_mint.key(),
            ctx.accounts.admin.key(),
            ctx.accounts.admin.key(),
            ctx.accounts.collection_md.key(),
            ctx.accounts.admin.key(),
            Some(0),
        );

        invoke(
            &collection_master_edition_ix,
            &create_collection_master_edition_accounts,
        )?;

        // set config
        // TODO: set account size properly
        // TODO: validate params
        let config = &mut ctx.accounts.config;
        config.name = params.name;
        config.threshold = params.threshold;
        config.collection_metadata_uri = params.collection_metadata_uri;
        config.item_metadata_uri = params.item_metadata_uri;

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
    #[account(init, payer = admin, seeds = [config.key().as_ref()], bump, mint::decimals = 0, mint::authority = admin)]
    pub collection_mint: Account<'info, token::Mint>,

    #[account(init, payer = admin, associated_token::mint = collection_mint, associated_token::authority = admin)]
    pub collection_mint_ata: Account<'info, token::TokenAccount>,

    /// CHECK: todo
    #[account(mut, seeds = [b"metadata", mpl_token_metadata::ID.as_ref(), collection_mint.key().as_ref()], bump, seeds::program = mpl_token_metadata::ID)]
    pub collection_md: AccountInfo<'info>,

    /// CHECK: todo
    #[account(mut, seeds = [b"metadata", mpl_token_metadata::ID.as_ref(), collection_mint.key().as_ref(), b"edition"], bump, seeds::program = mpl_token_metadata::ID)]
    pub collection_me: AccountInfo<'info>,

    #[account(init, space = 1024, payer = admin, seeds = [admin.key().as_ref(), params.name.as_bytes()], bump)]
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
    pub collection_metadata_uri: String,
    pub item_metadata_uri: String,
}

#[derive(AnchorSerialize, AnchorDeserialize)]
pub struct CreateRewardPlanParams {
    pub name: String,
    pub threshold: u64,
    pub collection_metadata_uri: String,
    pub item_metadata_uri: String,
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
