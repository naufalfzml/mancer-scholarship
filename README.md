# Solana Crowdfunding

A crowdfunding smart contract on Solana built with Anchor. Creators can launch campaigns, accept donations into a PDA vault, and either claim funds (if the goal is met) or donors can get refunds (if it fails).

## Program ID

```
BJGqnLChib5nebgzAkLuTDAcddSp9dEYjEMj86XRqTLj
```

## Instructions

| Instruction | Description |
|---|---|
| `create_campaign` | Create a new campaign with goal, deadline, title, and description |
| `contribute_campaign` | Donate SOL to a campaign (stored in PDA vault) |
| `withdraw` | Creator claims funds after goal reached + deadline passed |
| `refund` | Donor gets money back if campaign failed or was cancelled |
| `cancel_campaign` | Creator cancels campaign, enabling immediate refunds |

## Architecture

```
programs/solana-crowdfunding/src/
├── lib.rs              # Program entrypoint and instruction routing
├── state.rs            # Campaign and Contribution account structs
├── error.rs            # Custom error codes
├── constants.rs        # Seed constants
└── instructions/
    ├── create_campaign.rs
    ├── contribute.rs
    ├── withdraw.rs
    ├── refund.rs
    └── cancel_campaign.rs
```

### PDA Accounts

- **Vault**: `[b"vault", campaign_pubkey]` — holds all contributed SOL
- **Contribution**: `[b"contribution", campaign_pubkey, donor_pubkey]` — tracks per-donor contributions

### Campaign Lifecycle

```
Created → Active (accepting contributions)
  ├── Goal reached + deadline passed → Creator withdraws
  ├── Goal NOT reached + deadline passed → Donors refund
  └── Creator cancels → Donors refund immediately
```

## Setup

```bash
yarn install
anchor build
anchor keys sync
```

## Testing

Start a local validator in one terminal:

```bash
solana-test-validator --reset
```

Run tests in another terminal:

```bash
anchor test --skip-local-validator
```

## Deploy

```bash
# Localnet
anchor deploy

# Devnet
solana config set --url devnet
anchor deploy --provider.cluster devnet
```

## Tech Stack

- Anchor 1.0.0
- Solana CLI 2.2+
- TypeScript (tests)
