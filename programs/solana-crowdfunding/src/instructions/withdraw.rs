use anchor_lang::prelude::*;
use crate::state::Campaign;

#[derive(Accounts)]
pub struct Withdraw<'info> {
    #[account(mut)]
    pub campaign: Account<'info, Campaign>,

    #[account(mut)]
    pub creator: Signer<'info>,

    #[account(
        mut,
        seeds = [b"vault", campaign.key().as_ref()],
        bump,
    )]
    pub vault: SystemAccount<'info>,

    pub system_program: Program<'info, System>,
}

pub fn handler(ctx: Context<Withdraw>) -> Result<()> {
    let campaign = &mut ctx.accounts.campaign;
    let clock = Clock::get()?;
    let current_time = clock.unix_timestamp;
    let amount = ctx.accounts.vault.lamports();

    require!(campaign.raised >= campaign.goal);
    require!(current_time >= campaign.deadline);
    require!(ctx.accounts.creator.key() == campaign.creator);
    require!(campaign.claimed == false);

    system_program::transfer(
        CpiContext::new_with_signer(
            ctx.accounts.system_program.to_account_info(),
            system_program::Transfer {
                from: ctx.accounts.vault.to_account_info(),
                to: ctx.accounts.creator.to_account_info(),
            },
            &[&[b"vault", ctx.accounts.campaign.key().as_ref(), &[ctx.bumps.vault]]],
        ),
        amount,
    )?;

    campaign.claimed = true;

    Ok(())
}