use anchor_lang::prelude::*;
use crate::state::{Campaign, Contribution};

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

pub fn handler(ctx: Context<Contribute>, amount: u64) -> Result<()> {
    let campaign = &mut ctx.accounts.campaign;
    let contribution = &mut ctx.accounts.contribution;

    system_program::transfer(
        CpiContext::new(
            ctx.accounts.system_program.to_account_info(),
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
    contribution.campaign = ctx.accounts.campaign.key();

    Ok(())
}