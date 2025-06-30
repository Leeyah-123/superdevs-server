# Solana HTTP Server

A Rust-based HTTP server exposing Solana-related endpoints for blockchain operations.

## Features

- **Keypair Operations**: Generate, import, and manage Solana keypairs
- **SPL Token Management**: Create tokens, manage accounts, and check balances
- **Message Signing/Verification**: Securely sign and verify messages using Solana keypairs
- **Transaction Construction**: Build valid Solana transactions and instructions

## Setup

1. Clone this repository
2. Copy the `.env.example` file to `.env` and adjust settings as needed
3. Build and run the server:

```bash
cargo build
cargo run
```

## API Endpoints

### Health Check
- `GET /health` - Check server status

### Keypair Operations
- `GET /keypair/generate` - Generate a new random keypair
- `POST /keypair/from_seed` - Create keypair from seed bytes
- `POST /keypair/from_bytes` - Create keypair from raw bytes
- `POST /keypair/from_base58` - Create keypair from base58 string

### Message Operations
- `POST /message/sign` - Sign a message using a keypair
- `POST /message/verify` - Verify a message signature

### Token Operations
- `GET /token/account/{token_account}` - Get token account details
- `GET /token/balance/{token_account}` - Get token balance
- `POST /token/create_instructions` - Get instructions to create a new token
- `POST /token/create_associated_account` - Create associated token account
- `GET /token/mint/{mint_pubkey}` - Get mint account details

### Instruction Operations
- `POST /instruction/transfer` - Create SOL transfer instruction
- `POST /instruction/custom` - Create custom program instruction

## Environment Variables

- `PORT` - Server port (default: 8080)
- `BIND_ADDRESS` - Server bind address (default: 127.0.0.1)
- `RUST_LOG` - Logging level (default: info)
- `SOLANA_RPC_URL` - Solana RPC endpoint

## License

MIT
