use anchor_lang::prelude::*;
use anchor_spl::{self, associated_token, token};
use rewards::cpi::accounts::Reward;
use rewards::cpi::reward;
use rewards::program::Rewards;
use rewards::RewardParams;

declare_id!("HmbTLCmaGvZhKnn1Zfa1JVnp7vkMV4DYVxPLWBVoN65L");

#[program]
pub mod rewards_tester {
    use super::*;

    pub fn buy(ctx: Context<Buy>, name: String, admin: Pubkey) -> Result<()> {

        let reward_accounts = Reward {
            mint: ctx.accounts.mint.to_account_info(),
            config: ctx.accounts.config.to_account_info(),
            user: ctx.accounts.user.to_account_info(),
            user_ata: ctx.accounts.user_ata.to_account_info(),
            instructions: ctx.accounts.instructions.to_account_info(),
            system_program: ctx.accounts.system_program.to_account_info(),
            rent: ctx.accounts.rent.to_account_info(),
            token_program: ctx.accounts.token_program.to_account_info(),
            associated_token_program: ctx.accounts.associated_token_program.to_account_info(),
        };

        let reward_params = RewardParams {
            name: name,
            admin: admin,
            amount: 1,
        };

        let result = reward(CpiContext::new(ctx.accounts.reward_program.to_account_info(), reward_accounts), reward_params)?;

        // log if customer got rewarded by the program or not
        let rewarded = result.get();
        if rewarded {
            msg!("customer rewarded");
        } else {
            msg!("customer ineligible");
        }

        Ok(())
    }
}

#[derive(Accounts)]
#[instruction(name: String, admin: Pubkey)]
pub struct Buy<'info> {
    #[account(mut)]
    pub mint: Account<'info, token::Mint>,

    #[account()]
    pub config: Account<'info, rewards::RewardPlanConfig>,

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
    pub reward_program: Program<'info, Rewards>,
}
