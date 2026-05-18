# On-Chain Certificate & Skill Portfolio

A Soroban smart contract on Stellar blockchain for issuing tamper-proof
skill certificates and academic credentials — fully verifiable on-chain
without any central authority.

## Problem Statement

Certificate fraud is a real problem in Indonesia. Employers have no
reliable way to verify whether a diploma or skill certificate is genuine.
This contract solves that by anchoring every certificate permanently on
the Stellar blockchain — anyone can verify instantly, no middleman needed.

## Smart Contract ID (Testnet)

```
CC6SAMULG2QN6LMZ46A7JI2HIBYAPO3ADUHV4JFILWVLBTOBNGW6TNK6
```

## Features

- **Issuer Registry** — Admin registers trusted institutions (universities, bootcamps)
- **Issue Certificate** — Registered issuers mint a verifiable certificate to a recipient wallet
- **Revoke Certificate** — Original issuer can revoke a certificate if fraud is detected
- **Verify Certificate** — Anyone can verify a certificate by ID
- **Portfolio View** — See all certificates owned by a wallet address
- **Employer Check** — Instantly check if a candidate holds a valid certificate for a specific skill

## Contract Functions

### Setup
| Function | Access | Description |
|---|---|---|
| `initialize(admin)` | One-time | Set contract admin on deploy |

### Issuer Management
| Function | Access | Description |
|---|---|---|
| `add_issuer(issuer)` | Admin only | Register a trusted institution |
| `remove_issuer(issuer)` | Admin only | Revoke issuer status |
| `is_registered_issuer(issuer)` | Public | Check if address is a valid issuer |

### Certificates
| Function | Access | Description |
|---|---|---|
| `issue_certificate(...)` | Registered issuer | Mint a new certificate |
| `revoke_certificate(issuer, id)` | Original issuer | Invalidate a certificate |

### Verification
| Function | Access | Description |
|---|---|---|
| `verify_certificate(id)` | Public | Get certificate details and validity status |
| `get_portfolio(recipient)` | Public | All certificates owned by a wallet |
| `get_portfolio_summary(recipient)` | Public | Total and valid certificate count |
| `get_issued_by(issuer)` | Public | All certificates issued by an institution |
| `has_valid_certificate(recipient, title)` | Public | Check if wallet holds a valid skill cert |

## How It Works

```
1. Admin deploys contract and calls initialize(admin_wallet)
2. Admin registers Dicoding → add_issuer(dicoding_wallet)
3. Dicoding issues a cert → issue_certificate(...) → returns cert_id
4. Employer checks → has_valid_certificate(candidate_wallet, "React Developer") → true
5. Fraud detected → revoke_certificate(cert_id) → certificate marked invalid
```

## Project Structure

```
soroban-certificate-onchain/
├── Cargo.toml
└── src/
    ├── lib.rs       # Main smart contract
    └── test.rs      # Unit tests
```

## Tech Stack

- Language: Rust
- SDK: soroban-sdk v21
- Network: Stellar Testnet
- IDE: Soroban Studio

## AI Assistance

This project was developed with the assistance of Claude (Anthropic)
for smart contract architecture, Rust code generation, and test writing.

## License

MIT