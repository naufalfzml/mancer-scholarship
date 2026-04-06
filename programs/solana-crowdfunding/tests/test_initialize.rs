use anchor_lang::{
    solana_program::{clock::Clock, instruction::Instruction, system_program},
    InstructionData, ToAccountMetas,
};
use litesvm::LiteSVM;
use solana_keypair::Keypair;
use solana_message::{Message, VersionedMessage};
use solana_signer::Signer;
use solana_transaction::versioned::VersionedTransaction;

type Pubkey = anchor_lang::prelude::Pubkey;

const LAMPORTS_PER_SOL: u64 = 1_000_000_000;

// ── Helpers ──

fn setup() -> (LiteSVM, Keypair) {
    let mut svm = LiteSVM::new();
    let bytes = include_bytes!("../../../target/deploy/solana_crowdfunding.so");
    svm.add_program(solana_crowdfunding::id(), bytes).unwrap();

    let creator = Keypair::new();
    svm.airdrop(&creator.pubkey(), 100 * LAMPORTS_PER_SOL)
        .unwrap();

    (svm, creator)
}

fn vault_pda(campaign: &Pubkey) -> Pubkey {
    Pubkey::find_program_address(&[b"vault", campaign.as_ref()], &solana_crowdfunding::id()).0
}

fn contribution_pda(campaign: &Pubkey, donor: &Pubkey) -> Pubkey {
    Pubkey::find_program_address(
        &[b"contribution", campaign.as_ref(), donor.as_ref()],
        &solana_crowdfunding::id(),
    )
    .0
}

fn send_tx(svm: &mut LiteSVM, ixs: &[Instruction], signers: &[&Keypair]) -> bool {
    let blockhash = svm.latest_blockhash();
    let msg = Message::new_with_blockhash(ixs, Some(&signers[0].pubkey()), &blockhash);
    let tx = VersionedTransaction::try_new(VersionedMessage::Legacy(msg), signers).unwrap();
    svm.send_transaction(tx).is_ok()
}

fn current_time(svm: &mut LiteSVM) -> i64 {
    let clock: Clock = svm.get_sysvar();
    clock.unix_timestamp
}

fn advance_clock(svm: &mut LiteSVM, unix_timestamp: i64) {
    let mut clock: Clock = svm.get_sysvar();
    clock.unix_timestamp = unix_timestamp;
    svm.set_sysvar(&clock);
}

// ── Instruction builders ──

fn ix_create_campaign(
    creator: &Keypair,
    campaign: &Keypair,
    goal: u64,
    deadline: i64,
    title: &str,
    description: &str,
) -> Instruction {
    Instruction::new_with_bytes(
        solana_crowdfunding::id(),
        &solana_crowdfunding::instruction::CreateCampaign {
            goal,
            deadline,
            title: title.to_string(),
            description: description.to_string(),
        }
        .data(),
        solana_crowdfunding::accounts::CreateCampaign {
            creator: creator.pubkey(),
            campaign: campaign.pubkey(),
            system_program: system_program::id(),
        }
        .to_account_metas(None),
    )
}

fn ix_contribute(donor: &Keypair, campaign: &Pubkey, amount: u64) -> Instruction {
    Instruction::new_with_bytes(
        solana_crowdfunding::id(),
        &solana_crowdfunding::instruction::ContributeCampaign { amount }.data(),
        solana_crowdfunding::accounts::Contribute {
            campaign: *campaign,
            donor: donor.pubkey(),
            contribution: contribution_pda(campaign, &donor.pubkey()),
            system_program: system_program::id(),
            vault: vault_pda(campaign),
        }
        .to_account_metas(None),
    )
}

fn ix_withdraw(creator: &Keypair, campaign: &Pubkey) -> Instruction {
    Instruction::new_with_bytes(
        solana_crowdfunding::id(),
        &solana_crowdfunding::instruction::Withdraw {}.data(),
        solana_crowdfunding::accounts::Withdraw {
            campaign: *campaign,
            creator: creator.pubkey(),
            vault: vault_pda(campaign),
            system_program: system_program::id(),
        }
        .to_account_metas(None),
    )
}

fn ix_refund(donor: &Keypair, campaign: &Pubkey) -> Instruction {
    Instruction::new_with_bytes(
        solana_crowdfunding::id(),
        &solana_crowdfunding::instruction::Refund {}.data(),
        solana_crowdfunding::accounts::Refund {
            campaign: *campaign,
            donor: donor.pubkey(),
            vault: vault_pda(campaign),
            contribution: contribution_pda(campaign, &donor.pubkey()),
            system_program: system_program::id(),
        }
        .to_account_metas(None),
    )
}

fn ix_cancel(creator: &Keypair, campaign: &Pubkey) -> Instruction {
    Instruction::new_with_bytes(
        solana_crowdfunding::id(),
        &solana_crowdfunding::instruction::CancelCampaign {}.data(),
        solana_crowdfunding::accounts::CancelCampaign {
            campaign: *campaign,
            creator: creator.pubkey(),
        }
        .to_account_metas(None),
    )
}

// ── Create Campaign Tests ──

#[test]
fn test_create_campaign() {
    let (mut svm, creator) = setup();
    let campaign = Keypair::new();
    let deadline = current_time(&mut svm) + 1000;

    let ix = ix_create_campaign(&creator, &campaign, 10 * LAMPORTS_PER_SOL, deadline, "Test", "A test campaign");
    assert!(send_tx(&mut svm, &[ix], &[&creator, &campaign]));
}

#[test]
fn test_create_campaign_deadline_in_past() {
    let (mut svm, creator) = setup();
    let campaign = Keypair::new();
    let deadline = current_time(&mut svm) - 100;

    let ix = ix_create_campaign(&creator, &campaign, 10 * LAMPORTS_PER_SOL, deadline, "Test", "Desc");
    assert!(!send_tx(&mut svm, &[ix], &[&creator, &campaign]));
}

#[test]
fn test_create_campaign_goal_zero() {
    let (mut svm, creator) = setup();
    let campaign = Keypair::new();
    let deadline = current_time(&mut svm) + 1000;

    let ix = ix_create_campaign(&creator, &campaign, 0, deadline, "Test", "Desc");
    assert!(!send_tx(&mut svm, &[ix], &[&creator, &campaign]));
}

// ── Contribute Tests ──

#[test]
fn test_contribute() {
    let (mut svm, creator) = setup();
    let campaign = Keypair::new();
    let donor = Keypair::new();
    svm.airdrop(&donor.pubkey(), 100 * LAMPORTS_PER_SOL).unwrap();

    let deadline = current_time(&mut svm) + 1000;
    let ix_create = ix_create_campaign(&creator, &campaign, 10 * LAMPORTS_PER_SOL, deadline, "Test", "Desc");
    assert!(send_tx(&mut svm, &[ix_create], &[&creator, &campaign]));

    let ix = ix_contribute(&donor, &campaign.pubkey(), 5 * LAMPORTS_PER_SOL);
    assert!(send_tx(&mut svm, &[ix], &[&donor]));
}

#[test]
fn test_contribute_multiple_times() {
    let (mut svm, creator) = setup();
    let campaign = Keypair::new();
    let donor = Keypair::new();
    svm.airdrop(&donor.pubkey(), 100 * LAMPORTS_PER_SOL).unwrap();

    let deadline = current_time(&mut svm) + 1000;
    let ix_create = ix_create_campaign(&creator, &campaign, 10 * LAMPORTS_PER_SOL, deadline, "Test", "Desc");
    assert!(send_tx(&mut svm, &[ix_create], &[&creator, &campaign]));

    let ix1 = ix_contribute(&donor, &campaign.pubkey(), 3 * LAMPORTS_PER_SOL);
    assert!(send_tx(&mut svm, &[ix1], &[&donor]));

    let ix2 = ix_contribute(&donor, &campaign.pubkey(), 2 * LAMPORTS_PER_SOL);
    assert!(send_tx(&mut svm, &[ix2], &[&donor]));
}

#[test]
fn test_contribute_zero_amount() {
    let (mut svm, creator) = setup();
    let campaign = Keypair::new();
    let donor = Keypair::new();
    svm.airdrop(&donor.pubkey(), 100 * LAMPORTS_PER_SOL).unwrap();

    let deadline = current_time(&mut svm) + 1000;
    let ix_create = ix_create_campaign(&creator, &campaign, 10 * LAMPORTS_PER_SOL, deadline, "Test", "Desc");
    assert!(send_tx(&mut svm, &[ix_create], &[&creator, &campaign]));

    let ix = ix_contribute(&donor, &campaign.pubkey(), 0);
    assert!(!send_tx(&mut svm, &[ix], &[&donor]));
}

#[test]
fn test_contribute_after_deadline() {
    let (mut svm, creator) = setup();
    let campaign = Keypair::new();
    let donor = Keypair::new();
    svm.airdrop(&donor.pubkey(), 100 * LAMPORTS_PER_SOL).unwrap();

    let deadline = current_time(&mut svm) + 100;
    let ix_create = ix_create_campaign(&creator, &campaign, 10 * LAMPORTS_PER_SOL, deadline, "Test", "Desc");
    assert!(send_tx(&mut svm, &[ix_create], &[&creator, &campaign]));

    advance_clock(&mut svm, deadline + 1);

    let ix = ix_contribute(&donor, &campaign.pubkey(), 5 * LAMPORTS_PER_SOL);
    assert!(!send_tx(&mut svm, &[ix], &[&donor]));
}

#[test]
fn test_contribute_cancelled_campaign() {
    let (mut svm, creator) = setup();
    let campaign = Keypair::new();
    let donor = Keypair::new();
    svm.airdrop(&donor.pubkey(), 100 * LAMPORTS_PER_SOL).unwrap();

    let deadline = current_time(&mut svm) + 1000;
    let ix_create = ix_create_campaign(&creator, &campaign, 10 * LAMPORTS_PER_SOL, deadline, "Test", "Desc");
    assert!(send_tx(&mut svm, &[ix_create], &[&creator, &campaign]));

    let ix_cancel = ix_cancel(&creator, &campaign.pubkey());
    assert!(send_tx(&mut svm, &[ix_cancel], &[&creator]));

    let ix = ix_contribute(&donor, &campaign.pubkey(), 5 * LAMPORTS_PER_SOL);
    assert!(!send_tx(&mut svm, &[ix], &[&donor]));
}

// ── Withdraw Tests ──

#[test]
fn test_withdraw_success() {
    let (mut svm, creator) = setup();
    let campaign = Keypair::new();
    let donor = Keypair::new();
    svm.airdrop(&donor.pubkey(), 100 * LAMPORTS_PER_SOL).unwrap();

    let deadline = current_time(&mut svm) + 100;
    let goal = 10 * LAMPORTS_PER_SOL;

    let ix_create = ix_create_campaign(&creator, &campaign, goal, deadline, "Test", "Desc");
    assert!(send_tx(&mut svm, &[ix_create], &[&creator, &campaign]));

    let ix_contrib = ix_contribute(&donor, &campaign.pubkey(), goal);
    assert!(send_tx(&mut svm, &[ix_contrib], &[&donor]));

    advance_clock(&mut svm, deadline + 1);

    let ix = ix_withdraw(&creator, &campaign.pubkey());
    assert!(send_tx(&mut svm, &[ix], &[&creator]));
}

#[test]
fn test_withdraw_before_deadline() {
    let (mut svm, creator) = setup();
    let campaign = Keypair::new();
    let donor = Keypair::new();
    svm.airdrop(&donor.pubkey(), 100 * LAMPORTS_PER_SOL).unwrap();

    let deadline = current_time(&mut svm) + 10000;
    let goal = 10 * LAMPORTS_PER_SOL;

    let ix_create = ix_create_campaign(&creator, &campaign, goal, deadline, "Test", "Desc");
    assert!(send_tx(&mut svm, &[ix_create], &[&creator, &campaign]));

    let ix_contrib = ix_contribute(&donor, &campaign.pubkey(), goal);
    assert!(send_tx(&mut svm, &[ix_contrib], &[&donor]));

    let ix = ix_withdraw(&creator, &campaign.pubkey());
    assert!(!send_tx(&mut svm, &[ix], &[&creator]));
}

#[test]
fn test_withdraw_goal_not_reached() {
    let (mut svm, creator) = setup();
    let campaign = Keypair::new();
    let donor = Keypair::new();
    svm.airdrop(&donor.pubkey(), 100 * LAMPORTS_PER_SOL).unwrap();

    let deadline = current_time(&mut svm) + 100;
    let goal = 10 * LAMPORTS_PER_SOL;

    let ix_create = ix_create_campaign(&creator, &campaign, goal, deadline, "Test", "Desc");
    assert!(send_tx(&mut svm, &[ix_create], &[&creator, &campaign]));

    let ix_contrib = ix_contribute(&donor, &campaign.pubkey(), 5 * LAMPORTS_PER_SOL);
    assert!(send_tx(&mut svm, &[ix_contrib], &[&donor]));

    advance_clock(&mut svm, deadline + 1);

    let ix = ix_withdraw(&creator, &campaign.pubkey());
    assert!(!send_tx(&mut svm, &[ix], &[&creator]));
}

#[test]
fn test_withdraw_not_creator() {
    let (mut svm, creator) = setup();
    let campaign = Keypair::new();
    let donor = Keypair::new();
    let random = Keypair::new();
    svm.airdrop(&donor.pubkey(), 100 * LAMPORTS_PER_SOL).unwrap();
    svm.airdrop(&random.pubkey(), 100 * LAMPORTS_PER_SOL).unwrap();

    let deadline = current_time(&mut svm) + 100;
    let goal = 10 * LAMPORTS_PER_SOL;

    let ix_create = ix_create_campaign(&creator, &campaign, goal, deadline, "Test", "Desc");
    assert!(send_tx(&mut svm, &[ix_create], &[&creator, &campaign]));

    let ix_contrib = ix_contribute(&donor, &campaign.pubkey(), goal);
    assert!(send_tx(&mut svm, &[ix_contrib], &[&donor]));

    advance_clock(&mut svm, deadline + 1);

    let ix = ix_withdraw(&random, &campaign.pubkey());
    assert!(!send_tx(&mut svm, &[ix], &[&random]));
}

#[test]
fn test_withdraw_double() {
    let (mut svm, creator) = setup();
    let campaign = Keypair::new();
    let donor = Keypair::new();
    svm.airdrop(&donor.pubkey(), 100 * LAMPORTS_PER_SOL).unwrap();

    let deadline = current_time(&mut svm) + 100;
    let goal = 10 * LAMPORTS_PER_SOL;

    let ix_create = ix_create_campaign(&creator, &campaign, goal, deadline, "Test", "Desc");
    assert!(send_tx(&mut svm, &[ix_create], &[&creator, &campaign]));

    let ix_contrib = ix_contribute(&donor, &campaign.pubkey(), goal);
    assert!(send_tx(&mut svm, &[ix_contrib], &[&donor]));

    advance_clock(&mut svm, deadline + 1);

    let ix1 = ix_withdraw(&creator, &campaign.pubkey());
    assert!(send_tx(&mut svm, &[ix1], &[&creator]));

    let ix2 = ix_withdraw(&creator, &campaign.pubkey());
    assert!(!send_tx(&mut svm, &[ix2], &[&creator]));
}

#[test]
fn test_withdraw_cancelled_campaign() {
    let (mut svm, creator) = setup();
    let campaign = Keypair::new();
    let donor = Keypair::new();
    svm.airdrop(&donor.pubkey(), 100 * LAMPORTS_PER_SOL).unwrap();

    let deadline = current_time(&mut svm) + 100;
    let goal = 10 * LAMPORTS_PER_SOL;

    let ix_create = ix_create_campaign(&creator, &campaign, goal, deadline, "Test", "Desc");
    assert!(send_tx(&mut svm, &[ix_create], &[&creator, &campaign]));

    let ix_contrib = ix_contribute(&donor, &campaign.pubkey(), goal);
    assert!(send_tx(&mut svm, &[ix_contrib], &[&donor]));

    let ix_cancel = ix_cancel(&creator, &campaign.pubkey());
    assert!(send_tx(&mut svm, &[ix_cancel], &[&creator]));

    advance_clock(&mut svm, deadline + 1);

    let ix = ix_withdraw(&creator, &campaign.pubkey());
    assert!(!send_tx(&mut svm, &[ix], &[&creator]));
}

// ── Refund Tests ──

#[test]
fn test_refund_goal_not_reached() {
    let (mut svm, creator) = setup();
    let campaign = Keypair::new();
    let donor = Keypair::new();
    svm.airdrop(&donor.pubkey(), 100 * LAMPORTS_PER_SOL).unwrap();

    let deadline = current_time(&mut svm) + 100;
    let goal = 10 * LAMPORTS_PER_SOL;

    let ix_create = ix_create_campaign(&creator, &campaign, goal, deadline, "Test", "Desc");
    assert!(send_tx(&mut svm, &[ix_create], &[&creator, &campaign]));

    let ix_contrib = ix_contribute(&donor, &campaign.pubkey(), 5 * LAMPORTS_PER_SOL);
    assert!(send_tx(&mut svm, &[ix_contrib], &[&donor]));

    advance_clock(&mut svm, deadline + 1);

    let ix = ix_refund(&donor, &campaign.pubkey());
    assert!(send_tx(&mut svm, &[ix], &[&donor]));
}

#[test]
fn test_refund_before_deadline() {
    let (mut svm, creator) = setup();
    let campaign = Keypair::new();
    let donor = Keypair::new();
    svm.airdrop(&donor.pubkey(), 100 * LAMPORTS_PER_SOL).unwrap();

    let deadline = current_time(&mut svm) + 10000;
    let goal = 10 * LAMPORTS_PER_SOL;

    let ix_create = ix_create_campaign(&creator, &campaign, goal, deadline, "Test", "Desc");
    assert!(send_tx(&mut svm, &[ix_create], &[&creator, &campaign]));

    let ix_contrib = ix_contribute(&donor, &campaign.pubkey(), 5 * LAMPORTS_PER_SOL);
    assert!(send_tx(&mut svm, &[ix_contrib], &[&donor]));

    let ix = ix_refund(&donor, &campaign.pubkey());
    assert!(!send_tx(&mut svm, &[ix], &[&donor]));
}

#[test]
fn test_refund_after_cancel() {
    let (mut svm, creator) = setup();
    let campaign = Keypair::new();
    let donor = Keypair::new();
    svm.airdrop(&donor.pubkey(), 100 * LAMPORTS_PER_SOL).unwrap();

    let deadline = current_time(&mut svm) + 1000;

    let ix_create = ix_create_campaign(&creator, &campaign, 10 * LAMPORTS_PER_SOL, deadline, "Test", "Desc");
    assert!(send_tx(&mut svm, &[ix_create], &[&creator, &campaign]));

    let ix_contrib = ix_contribute(&donor, &campaign.pubkey(), 5 * LAMPORTS_PER_SOL);
    assert!(send_tx(&mut svm, &[ix_contrib], &[&donor]));

    let ix_cancel = ix_cancel(&creator, &campaign.pubkey());
    assert!(send_tx(&mut svm, &[ix_cancel], &[&creator]));

    // Refund should work immediately — no need to wait for deadline
    let ix = ix_refund(&donor, &campaign.pubkey());
    assert!(send_tx(&mut svm, &[ix], &[&donor]));
}

// ── Cancel Campaign Tests ──

#[test]
fn test_cancel_campaign() {
    let (mut svm, creator) = setup();
    let campaign = Keypair::new();
    let deadline = current_time(&mut svm) + 1000;

    let ix_create = ix_create_campaign(&creator, &campaign, 10 * LAMPORTS_PER_SOL, deadline, "Test", "Desc");
    assert!(send_tx(&mut svm, &[ix_create], &[&creator, &campaign]));

    let ix = ix_cancel(&creator, &campaign.pubkey());
    assert!(send_tx(&mut svm, &[ix], &[&creator]));
}

#[test]
fn test_cancel_not_creator() {
    let (mut svm, creator) = setup();
    let campaign = Keypair::new();
    let random = Keypair::new();
    svm.airdrop(&random.pubkey(), 100 * LAMPORTS_PER_SOL).unwrap();

    let deadline = current_time(&mut svm) + 1000;

    let ix_create = ix_create_campaign(&creator, &campaign, 10 * LAMPORTS_PER_SOL, deadline, "Test", "Desc");
    assert!(send_tx(&mut svm, &[ix_create], &[&creator, &campaign]));

    let ix = ix_cancel(&random, &campaign.pubkey());
    assert!(!send_tx(&mut svm, &[ix], &[&random]));
}

#[test]
fn test_cancel_double() {
    let (mut svm, creator) = setup();
    let campaign = Keypair::new();
    let deadline = current_time(&mut svm) + 1000;

    let ix_create = ix_create_campaign(&creator, &campaign, 10 * LAMPORTS_PER_SOL, deadline, "Test", "Desc");
    assert!(send_tx(&mut svm, &[ix_create], &[&creator, &campaign]));

    let ix1 = ix_cancel(&creator, &campaign.pubkey());
    assert!(send_tx(&mut svm, &[ix1], &[&creator]));

    let ix2 = ix_cancel(&creator, &campaign.pubkey());
    assert!(!send_tx(&mut svm, &[ix2], &[&creator]));
}
