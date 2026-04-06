use anchor_lang::prelude::*;
use crate::state::{Campaign, Contribution};
use crate::error::CrowdfundingError;
use anchor_lang::system_program;

#[derive(Accounts)]
pub struct Contribute<'info> {
    #[account(mut)]
    pub campaign: Account<'info, Campaign>,
    
    #[account(mut)]
    pub donor: Signer<'info>,

    #[account(
        init_if_needed,
        payer = donor,
        space = 8 + Contribution::INIT_SPACE,
        seeds = [b"contribution", campaign.key().as_ref(), donor.key().as_ref()],
        bump,
    )]
    pub contribution: Account<'info, Contribution>,
    pub system_program: Program<'info, System>,

    #[account(
        mut,
        seeds = [b"vault", campaign.key().as_ref()],
        bump,
    )]
    pub vault: SystemAccount<'info>
}

pub fn contribute_handler(ctx: Context<Contribute>, amount: u64) -> Result<()> {
    let campaign_key = ctx.accounts.campaign.key();
    let campaign = &mut ctx.accounts.campaign;
    let contribution = &mut ctx.accounts.contribution;
    let clock = Clock::get()?;
    let current_time = clock.unix_timestamp;

    require!(!campaign.cancelled, CrowdfundingError::CampaignCancelled);
    require!(current_time < campaign.deadline, CrowdfundingError::DeadlinePassed);

    system_program::transfer(
        CpiContext::new(
            ctx.accounts.system_program.key(),
            system_program::Transfer {
                from: ctx.accounts.donor.to_account_info(),
                to: ctx.accounts.vault.to_account_info(),
            },
        ),
        amount,
    )?;

    campaign.raised += amount;
    contribution.donor = ctx.accounts.donor.key();
    contribution.amount += amount;
    contribution.campaign = campaign_key;

    msg!("Contributed: {} lamports, total={}", amount, campaign.raised);

    Ok(())
}