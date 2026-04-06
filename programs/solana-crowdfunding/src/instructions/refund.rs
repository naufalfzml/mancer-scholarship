use anchor_lang::prelude::*;
use crate::state::{Campaign, Contribution};
use crate::error::CrowdfundingError;
use anchor_lang::system_program;

#[derive(Accounts)]
pub struct Refund<'info> {
    #[account(mut)]
    pub campaign: Account<'info, Campaign>,

    #[account(mut)]
    pub donor: Signer<'info>,

    #[account(
        mut, 
        seeds= [b"vault", campaign.key().as_ref()],
        bump,
    )]
    pub vault: SystemAccount<'info>,

    #[account(
        mut,
        seeds = [b"contribution", campaign.key().as_ref(), donor.key().as_ref()],
        bump,
    )]
    pub contribution: Account<'info, Contribution>,

    pub system_program:Program<'info, System>,
}

pub fn refund_handler(ctx: Context<Refund>) -> Result<()> {
    let campaign_key = ctx.accounts.campaign.key();
    let campaign = &mut ctx.accounts.campaign;
    let contribution = &mut ctx.accounts.contribution;
    let amount = contribution.amount;

    let clock = Clock::get()?;
    let current_time = clock.unix_timestamp;

    require!(campaign.raised < campaign.goal, CrowdfundingError::GoalReached);
    require!(current_time >= campaign.deadline, CrowdfundingError::DeadlineNotPassed);

    system_program::transfer(
        CpiContext::new_with_signer(
            ctx.accounts.system_program.key(),
            system_program::Transfer {
                from: ctx.accounts.vault.to_account_info(),
                to: ctx.accounts.donor.to_account_info(),
            },
            &[&[b"vault", campaign_key.as_ref(), &[ctx.bumps.vault]]],
        ),
        amount,
    )?;
    contribution.amount = 0;
    campaign.raised -= amount;

    msg!("Refunded: {} lamports", amount);
    Ok(())
}