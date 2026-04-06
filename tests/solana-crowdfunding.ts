import * as anchor from "@anchor-lang/core";
import { Program } from "@anchor-lang/core";
import { SolanaCrowdfunding } from "../target/types/solana_crowdfunding";
import { expect } from "chai";
import BN from "bn.js";

const sleep = (ms: number) => new Promise((r) => setTimeout(r, ms));

describe("solana-crowdfunding", () => {
  const provider = anchor.AnchorProvider.env();
  anchor.setProvider(provider);
  const program = anchor.workspace.SolanaCrowdfunding as Program<SolanaCrowdfunding>;

  const { Keypair, PublicKey, SystemProgram, LAMPORTS_PER_SOL } = anchor.web3;

  async function getCurrentTime(): Promise<number> {
    const slot = await provider.connection.getSlot();
    const time = await provider.connection.getBlockTime(slot);
    return time!;
  }

  async function airdrop(pubkey: anchor.web3.PublicKey, sol: number = 100) {
    const sig = await provider.connection.requestAirdrop(
      pubkey,
      sol * LAMPORTS_PER_SOL
    );
    await provider.connection.confirmTransaction(sig);
  }

  async function createCampaign(
    creator: anchor.web3.Keypair | null,
    goal: number,
    deadlineOffset: number,
    title: string = "Test Campaign",
    description: string = "A test campaign"
  ) {
    const campaign = Keypair.generate();
    const currentTime = await getCurrentTime();
    const deadline = new BN(currentTime + deadlineOffset);

    const tx = program.methods
      .createCampaign(new BN(goal), deadline, title, description)
      .accounts({
        creator: creator
          ? creator.publicKey
          : provider.wallet.publicKey,
        campaign: campaign.publicKey,
        systemProgram: SystemProgram.programId,
      });

    if (creator) {
      await tx.signers([creator, campaign]).rpc();
    } else {
      await tx.signers([campaign]).rpc();
    }

    return { campaign, deadline };
  }

  // ── Create Campaign ──

  describe("create_campaign", () => {
    it("creates a campaign successfully", async () => {
      const { campaign } = await createCampaign(
        null,
        10 * LAMPORTS_PER_SOL,
        3600
      );

      const acc = await program.account.campaign.fetch(campaign.publicKey);
      expect(acc.creator.toString()).to.equal(
        provider.wallet.publicKey.toString()
      );
      expect(acc.goal.toNumber()).to.equal(10 * LAMPORTS_PER_SOL);
      expect(acc.raised.toNumber()).to.equal(0);
      expect(acc.claimed).to.be.false;
      expect(acc.cancelled).to.be.false;
      expect(acc.title).to.equal("Test Campaign");
      expect(acc.description).to.equal("A test campaign");
    });

    it("fails with deadline in past", async () => {
      try {
        await createCampaign(null, 10 * LAMPORTS_PER_SOL, -100);
        expect.fail("Should have failed");
      } catch (err: any) {
        expect(err.toString()).to.include("DeadlineInPast");
      }
    });

    it("fails with goal zero", async () => {
      try {
        await createCampaign(null, 0, 3600);
        expect.fail("Should have failed");
      } catch (err: any) {
        expect(err.toString()).to.include("GoalZero");
      }
    });
  });

  // ── Contribute ──

  describe("contribute_campaign", () => {
    it("contributes successfully", async () => {
      const { campaign } = await createCampaign(
        null,
        10 * LAMPORTS_PER_SOL,
        3600
      );
      const donor = Keypair.generate();
      await airdrop(donor.publicKey);

      const [contributionPda] = PublicKey.findProgramAddressSync(
        [
          Buffer.from("contribution"),
          campaign.publicKey.toBuffer(),
          donor.publicKey.toBuffer(),
        ],
        program.programId
      );
      const [vaultPda] = PublicKey.findProgramAddressSync(
        [Buffer.from("vault"), campaign.publicKey.toBuffer()],
        program.programId
      );

      await program.methods
        .contributeCampaign(new BN(5 * LAMPORTS_PER_SOL))
        .accounts({
          campaign: campaign.publicKey,
          donor: donor.publicKey,
          contribution: contributionPda,
          systemProgram: SystemProgram.programId,
          vault: vaultPda,
        })
        .signers([donor])
        .rpc();

      const acc = await program.account.campaign.fetch(campaign.publicKey);
      expect(acc.raised.toNumber()).to.equal(5 * LAMPORTS_PER_SOL);
    });

    it("contributes multiple times", async () => {
      const { campaign } = await createCampaign(
        null,
        10 * LAMPORTS_PER_SOL,
        3600
      );
      const donor = Keypair.generate();
      await airdrop(donor.publicKey);

      const [contributionPda] = PublicKey.findProgramAddressSync(
        [
          Buffer.from("contribution"),
          campaign.publicKey.toBuffer(),
          donor.publicKey.toBuffer(),
        ],
        program.programId
      );
      const [vaultPda] = PublicKey.findProgramAddressSync(
        [Buffer.from("vault"), campaign.publicKey.toBuffer()],
        program.programId
      );

      const accounts = {
        campaign: campaign.publicKey,
        donor: donor.publicKey,
        contribution: contributionPda,
        systemProgram: SystemProgram.programId,
        vault: vaultPda,
      };

      await program.methods
        .contributeCampaign(new BN(3 * LAMPORTS_PER_SOL))
        .accounts(accounts)
        .signers([donor])
        .rpc();

      await program.methods
        .contributeCampaign(new BN(2 * LAMPORTS_PER_SOL))
        .accounts(accounts)
        .signers([donor])
        .rpc();

      const acc = await program.account.campaign.fetch(campaign.publicKey);
      expect(acc.raised.toNumber()).to.equal(5 * LAMPORTS_PER_SOL);

      const contrib = await program.account.contribution.fetch(contributionPda);
      expect(contrib.amount.toNumber()).to.equal(5 * LAMPORTS_PER_SOL);
    });

    it("fails with zero amount", async () => {
      const { campaign } = await createCampaign(
        null,
        10 * LAMPORTS_PER_SOL,
        3600
      );
      const donor = Keypair.generate();
      await airdrop(donor.publicKey);

      const [contributionPda] = PublicKey.findProgramAddressSync(
        [
          Buffer.from("contribution"),
          campaign.publicKey.toBuffer(),
          donor.publicKey.toBuffer(),
        ],
        program.programId
      );
      const [vaultPda] = PublicKey.findProgramAddressSync(
        [Buffer.from("vault"), campaign.publicKey.toBuffer()],
        program.programId
      );

      try {
        await program.methods
          .contributeCampaign(new BN(0))
          .accounts({
            campaign: campaign.publicKey,
            donor: donor.publicKey,
            contribution: contributionPda,
            systemProgram: SystemProgram.programId,
            vault: vaultPda,
          })
          .signers([donor])
          .rpc();
        expect.fail("Should have failed");
      } catch (err: any) {
        expect(err.toString()).to.include("AmountZero");
      }
    });

    it("fails after deadline", async () => {
      const { campaign } = await createCampaign(
        null,
        10 * LAMPORTS_PER_SOL,
        3
      );
      const donor = Keypair.generate();
      await airdrop(donor.publicKey);

      await sleep(4000);

      const [contributionPda] = PublicKey.findProgramAddressSync(
        [
          Buffer.from("contribution"),
          campaign.publicKey.toBuffer(),
          donor.publicKey.toBuffer(),
        ],
        program.programId
      );
      const [vaultPda] = PublicKey.findProgramAddressSync(
        [Buffer.from("vault"), campaign.publicKey.toBuffer()],
        program.programId
      );

      try {
        await program.methods
          .contributeCampaign(new BN(5 * LAMPORTS_PER_SOL))
          .accounts({
            campaign: campaign.publicKey,
            donor: donor.publicKey,
            contribution: contributionPda,
            systemProgram: SystemProgram.programId,
            vault: vaultPda,
          })
          .signers([donor])
          .rpc();
        expect.fail("Should have failed");
      } catch (err: any) {
        expect(err.toString()).to.include("DeadlinePassed");
      }
    });

    it("fails on cancelled campaign", async () => {
      const { campaign } = await createCampaign(
        null,
        10 * LAMPORTS_PER_SOL,
        3600
      );

      await program.methods
        .cancelCampaign()
        .accounts({
          campaign: campaign.publicKey,
          creator: provider.wallet.publicKey,
        })
        .rpc();

      const donor = Keypair.generate();
      await airdrop(donor.publicKey);

      const [contributionPda] = PublicKey.findProgramAddressSync(
        [
          Buffer.from("contribution"),
          campaign.publicKey.toBuffer(),
          donor.publicKey.toBuffer(),
        ],
        program.programId
      );
      const [vaultPda] = PublicKey.findProgramAddressSync(
        [Buffer.from("vault"), campaign.publicKey.toBuffer()],
        program.programId
      );

      try {
        await program.methods
          .contributeCampaign(new BN(5 * LAMPORTS_PER_SOL))
          .accounts({
            campaign: campaign.publicKey,
            donor: donor.publicKey,
            contribution: contributionPda,
            systemProgram: SystemProgram.programId,
            vault: vaultPda,
          })
          .signers([donor])
          .rpc();
        expect.fail("Should have failed");
      } catch (err: any) {
        expect(err.toString()).to.include("CampaignCancelled");
      }
    });
  });

  // ── Withdraw ──

  describe("withdraw", () => {
    it("withdraws successfully after deadline with goal reached", async () => {
      const { campaign } = await createCampaign(
        null,
        10 * LAMPORTS_PER_SOL,
        3
      );
      const donor = Keypair.generate();
      await airdrop(donor.publicKey);

      const [contributionPda] = PublicKey.findProgramAddressSync(
        [
          Buffer.from("contribution"),
          campaign.publicKey.toBuffer(),
          donor.publicKey.toBuffer(),
        ],
        program.programId
      );
      const [vaultPda] = PublicKey.findProgramAddressSync(
        [Buffer.from("vault"), campaign.publicKey.toBuffer()],
        program.programId
      );

      await program.methods
        .contributeCampaign(new BN(10 * LAMPORTS_PER_SOL))
        .accounts({
          campaign: campaign.publicKey,
          donor: donor.publicKey,
          contribution: contributionPda,
          systemProgram: SystemProgram.programId,
          vault: vaultPda,
        })
        .signers([donor])
        .rpc();

      await sleep(4000);

      const balanceBefore = await provider.connection.getBalance(
        provider.wallet.publicKey
      );

      await program.methods
        .withdraw()
        .accounts({
          campaign: campaign.publicKey,
          creator: provider.wallet.publicKey,
          vault: vaultPda,
          systemProgram: SystemProgram.programId,
        })
        .rpc();

      const balanceAfter = await provider.connection.getBalance(
        provider.wallet.publicKey
      );
      expect(balanceAfter).to.be.greaterThan(balanceBefore);

      const acc = await program.account.campaign.fetch(campaign.publicKey);
      expect(acc.claimed).to.be.true;
    });

    it("fails before deadline", async () => {
      const { campaign } = await createCampaign(
        null,
        10 * LAMPORTS_PER_SOL,
        3600
      );
      const donor = Keypair.generate();
      await airdrop(donor.publicKey);

      const [contributionPda] = PublicKey.findProgramAddressSync(
        [
          Buffer.from("contribution"),
          campaign.publicKey.toBuffer(),
          donor.publicKey.toBuffer(),
        ],
        program.programId
      );
      const [vaultPda] = PublicKey.findProgramAddressSync(
        [Buffer.from("vault"), campaign.publicKey.toBuffer()],
        program.programId
      );

      await program.methods
        .contributeCampaign(new BN(10 * LAMPORTS_PER_SOL))
        .accounts({
          campaign: campaign.publicKey,
          donor: donor.publicKey,
          contribution: contributionPda,
          systemProgram: SystemProgram.programId,
          vault: vaultPda,
        })
        .signers([donor])
        .rpc();

      try {
        await program.methods
          .withdraw()
          .accounts({
            campaign: campaign.publicKey,
            creator: provider.wallet.publicKey,
            vault: vaultPda,
            systemProgram: SystemProgram.programId,
          })
          .rpc();
        expect.fail("Should have failed");
      } catch (err: any) {
        expect(err.toString()).to.include("DeadlineNotPassed");
      }
    });

    it("fails when goal not reached", async () => {
      const { campaign } = await createCampaign(
        null,
        10 * LAMPORTS_PER_SOL,
        3
      );
      const donor = Keypair.generate();
      await airdrop(donor.publicKey);

      const [contributionPda] = PublicKey.findProgramAddressSync(
        [
          Buffer.from("contribution"),
          campaign.publicKey.toBuffer(),
          donor.publicKey.toBuffer(),
        ],
        program.programId
      );
      const [vaultPda] = PublicKey.findProgramAddressSync(
        [Buffer.from("vault"), campaign.publicKey.toBuffer()],
        program.programId
      );

      await program.methods
        .contributeCampaign(new BN(5 * LAMPORTS_PER_SOL))
        .accounts({
          campaign: campaign.publicKey,
          donor: donor.publicKey,
          contribution: contributionPda,
          systemProgram: SystemProgram.programId,
          vault: vaultPda,
        })
        .signers([donor])
        .rpc();

      await sleep(4000);

      try {
        await program.methods
          .withdraw()
          .accounts({
            campaign: campaign.publicKey,
            creator: provider.wallet.publicKey,
            vault: vaultPda,
            systemProgram: SystemProgram.programId,
          })
          .rpc();
        expect.fail("Should have failed");
      } catch (err: any) {
        expect(err.toString()).to.include("GoalNotReached");
      }
    });

    it("fails when not creator", async () => {
      const { campaign } = await createCampaign(
        null,
        10 * LAMPORTS_PER_SOL,
        3
      );
      const donor = Keypair.generate();
      const random = Keypair.generate();
      await airdrop(donor.publicKey);
      await airdrop(random.publicKey);

      const [contributionPda] = PublicKey.findProgramAddressSync(
        [
          Buffer.from("contribution"),
          campaign.publicKey.toBuffer(),
          donor.publicKey.toBuffer(),
        ],
        program.programId
      );
      const [vaultPda] = PublicKey.findProgramAddressSync(
        [Buffer.from("vault"), campaign.publicKey.toBuffer()],
        program.programId
      );

      await program.methods
        .contributeCampaign(new BN(10 * LAMPORTS_PER_SOL))
        .accounts({
          campaign: campaign.publicKey,
          donor: donor.publicKey,
          contribution: contributionPda,
          systemProgram: SystemProgram.programId,
          vault: vaultPda,
        })
        .signers([donor])
        .rpc();

      await sleep(4000);

      try {
        await program.methods
          .withdraw()
          .accounts({
            campaign: campaign.publicKey,
            creator: random.publicKey,
            vault: vaultPda,
            systemProgram: SystemProgram.programId,
          })
          .signers([random])
          .rpc();
        expect.fail("Should have failed");
      } catch (err: any) {
        expect(err.toString()).to.include("NotCreator");
      }
    });

    it("fails on double withdrawal", async () => {
      const { campaign } = await createCampaign(
        null,
        10 * LAMPORTS_PER_SOL,
        3
      );
      const donor = Keypair.generate();
      await airdrop(donor.publicKey);

      const [contributionPda] = PublicKey.findProgramAddressSync(
        [
          Buffer.from("contribution"),
          campaign.publicKey.toBuffer(),
          donor.publicKey.toBuffer(),
        ],
        program.programId
      );
      const [vaultPda] = PublicKey.findProgramAddressSync(
        [Buffer.from("vault"), campaign.publicKey.toBuffer()],
        program.programId
      );

      await program.methods
        .contributeCampaign(new BN(10 * LAMPORTS_PER_SOL))
        .accounts({
          campaign: campaign.publicKey,
          donor: donor.publicKey,
          contribution: contributionPda,
          systemProgram: SystemProgram.programId,
          vault: vaultPda,
        })
        .signers([donor])
        .rpc();

      await sleep(4000);

      const withdrawAccounts = {
        campaign: campaign.publicKey,
        creator: provider.wallet.publicKey,
        vault: vaultPda,
        systemProgram: SystemProgram.programId,
      };

      await program.methods.withdraw().accounts(withdrawAccounts).rpc();

      try {
        await program.methods.withdraw().accounts(withdrawAccounts).rpc();
        expect.fail("Should have failed");
      } catch (err: any) {
        expect(err.toString()).to.include("AlreadyClaimed");
      }
    });

    it("fails on cancelled campaign", async () => {
      const { campaign } = await createCampaign(
        null,
        10 * LAMPORTS_PER_SOL,
        3
      );
      const donor = Keypair.generate();
      await airdrop(donor.publicKey);

      const [contributionPda] = PublicKey.findProgramAddressSync(
        [
          Buffer.from("contribution"),
          campaign.publicKey.toBuffer(),
          donor.publicKey.toBuffer(),
        ],
        program.programId
      );
      const [vaultPda] = PublicKey.findProgramAddressSync(
        [Buffer.from("vault"), campaign.publicKey.toBuffer()],
        program.programId
      );

      await program.methods
        .contributeCampaign(new BN(10 * LAMPORTS_PER_SOL))
        .accounts({
          campaign: campaign.publicKey,
          donor: donor.publicKey,
          contribution: contributionPda,
          systemProgram: SystemProgram.programId,
          vault: vaultPda,
        })
        .signers([donor])
        .rpc();

      await program.methods
        .cancelCampaign()
        .accounts({
          campaign: campaign.publicKey,
          creator: provider.wallet.publicKey,
        })
        .rpc();

      await sleep(4000);

      try {
        await program.methods
          .withdraw()
          .accounts({
            campaign: campaign.publicKey,
            creator: provider.wallet.publicKey,
            vault: vaultPda,
            systemProgram: SystemProgram.programId,
          })
          .rpc();
        expect.fail("Should have failed");
      } catch (err: any) {
        expect(err.toString()).to.include("CampaignCancelled");
      }
    });
  });

  // ── Refund ──

  describe("refund", () => {
    it("refunds when goal not reached after deadline", async () => {
      const { campaign } = await createCampaign(
        null,
        10 * LAMPORTS_PER_SOL,
        3
      );
      const donor = Keypair.generate();
      await airdrop(donor.publicKey);

      const [contributionPda] = PublicKey.findProgramAddressSync(
        [
          Buffer.from("contribution"),
          campaign.publicKey.toBuffer(),
          donor.publicKey.toBuffer(),
        ],
        program.programId
      );
      const [vaultPda] = PublicKey.findProgramAddressSync(
        [Buffer.from("vault"), campaign.publicKey.toBuffer()],
        program.programId
      );

      await program.methods
        .contributeCampaign(new BN(5 * LAMPORTS_PER_SOL))
        .accounts({
          campaign: campaign.publicKey,
          donor: donor.publicKey,
          contribution: contributionPda,
          systemProgram: SystemProgram.programId,
          vault: vaultPda,
        })
        .signers([donor])
        .rpc();

      await sleep(4000);

      const balanceBefore = await provider.connection.getBalance(
        donor.publicKey
      );

      await program.methods
        .refund()
        .accounts({
          campaign: campaign.publicKey,
          donor: donor.publicKey,
          vault: vaultPda,
          contribution: contributionPda,
          systemProgram: SystemProgram.programId,
        })
        .signers([donor])
        .rpc();

      const balanceAfter = await provider.connection.getBalance(
        donor.publicKey
      );
      expect(balanceAfter).to.be.greaterThan(balanceBefore);
    });

    it("fails before deadline", async () => {
      const { campaign } = await createCampaign(
        null,
        10 * LAMPORTS_PER_SOL,
        3600
      );
      const donor = Keypair.generate();
      await airdrop(donor.publicKey);

      const [contributionPda] = PublicKey.findProgramAddressSync(
        [
          Buffer.from("contribution"),
          campaign.publicKey.toBuffer(),
          donor.publicKey.toBuffer(),
        ],
        program.programId
      );
      const [vaultPda] = PublicKey.findProgramAddressSync(
        [Buffer.from("vault"), campaign.publicKey.toBuffer()],
        program.programId
      );

      await program.methods
        .contributeCampaign(new BN(5 * LAMPORTS_PER_SOL))
        .accounts({
          campaign: campaign.publicKey,
          donor: donor.publicKey,
          contribution: contributionPda,
          systemProgram: SystemProgram.programId,
          vault: vaultPda,
        })
        .signers([donor])
        .rpc();

      try {
        await program.methods
          .refund()
          .accounts({
            campaign: campaign.publicKey,
            donor: donor.publicKey,
            vault: vaultPda,
            contribution: contributionPda,
            systemProgram: SystemProgram.programId,
          })
          .signers([donor])
          .rpc();
        expect.fail("Should have failed");
      } catch (err: any) {
        expect(err.toString()).to.include("RefundNotAllowed");
      }
    });

    it("refunds immediately after cancel", async () => {
      const { campaign } = await createCampaign(
        null,
        10 * LAMPORTS_PER_SOL,
        3600
      );
      const donor = Keypair.generate();
      await airdrop(donor.publicKey);

      const [contributionPda] = PublicKey.findProgramAddressSync(
        [
          Buffer.from("contribution"),
          campaign.publicKey.toBuffer(),
          donor.publicKey.toBuffer(),
        ],
        program.programId
      );
      const [vaultPda] = PublicKey.findProgramAddressSync(
        [Buffer.from("vault"), campaign.publicKey.toBuffer()],
        program.programId
      );

      await program.methods
        .contributeCampaign(new BN(5 * LAMPORTS_PER_SOL))
        .accounts({
          campaign: campaign.publicKey,
          donor: donor.publicKey,
          contribution: contributionPda,
          systemProgram: SystemProgram.programId,
          vault: vaultPda,
        })
        .signers([donor])
        .rpc();

      await program.methods
        .cancelCampaign()
        .accounts({
          campaign: campaign.publicKey,
          creator: provider.wallet.publicKey,
        })
        .rpc();

      // No sleep needed - cancel allows immediate refund
      await program.methods
        .refund()
        .accounts({
          campaign: campaign.publicKey,
          donor: donor.publicKey,
          vault: vaultPda,
          contribution: contributionPda,
          systemProgram: SystemProgram.programId,
        })
        .signers([donor])
        .rpc();

      const contrib = await program.account.contribution.fetch(contributionPda);
      expect(contrib.amount.toNumber()).to.equal(0);
    });
  });

  // ── Cancel Campaign ──

  describe("cancel_campaign", () => {
    it("cancels successfully", async () => {
      const { campaign } = await createCampaign(
        null,
        10 * LAMPORTS_PER_SOL,
        3600
      );

      await program.methods
        .cancelCampaign()
        .accounts({
          campaign: campaign.publicKey,
          creator: provider.wallet.publicKey,
        })
        .rpc();

      const acc = await program.account.campaign.fetch(campaign.publicKey);
      expect(acc.cancelled).to.be.true;
    });

    it("fails when not creator", async () => {
      const { campaign } = await createCampaign(
        null,
        10 * LAMPORTS_PER_SOL,
        3600
      );
      const random = Keypair.generate();
      await airdrop(random.publicKey);

      try {
        await program.methods
          .cancelCampaign()
          .accounts({
            campaign: campaign.publicKey,
            creator: random.publicKey,
          })
          .signers([random])
          .rpc();
        expect.fail("Should have failed");
      } catch (err: any) {
        expect(err.toString()).to.include("NotCreator");
      }
    });

    it("fails on double cancel", async () => {
      const { campaign } = await createCampaign(
        null,
        10 * LAMPORTS_PER_SOL,
        3600
      );

      await program.methods
        .cancelCampaign()
        .accounts({
          campaign: campaign.publicKey,
          creator: provider.wallet.publicKey,
        })
        .rpc();

      try {
        await program.methods
          .cancelCampaign()
          .accounts({
            campaign: campaign.publicKey,
            creator: provider.wallet.publicKey,
          })
          .rpc();
        expect.fail("Should have failed");
      } catch (err: any) {
        expect(err.toString()).to.include("CampaignCancelled");
      }
    });
  });
});
