use actix_web::{HttpResponse, Responder, post, web};
use base64::{Engine as _, engine::general_purpose};
use serde::{Deserialize, Serialize};
use solana_sdk::{instruction::AccountMeta, pubkey::Pubkey, system_instruction};

use crate::response::{ApiResponse, ErrorResponse};
use spl_associated_token_account::get_associated_token_address;
use spl_token::instruction as token_instruction;
use std::str::FromStr;

use crate::error::{Result, ServerError};

pub mod routes {
    use super::*;
    use actix_web::web;

    pub fn configure(cfg: &mut web::ServiceConfig) {
        cfg.service(web::scope("/send").service(send_sol).service(send_token));
    }
}

#[derive(Deserialize)]
struct SendSolRequest {
    from: String,
    to: String,
    lamports: u64,
}

#[derive(Deserialize)]
struct SendTokenRequest {
    destination: String,
    mint: String,
    owner: String,
    amount: u64,
}

// #[derive(Serialize)]
// struct AccountMeta {
//     pubkey: String,
//     is_signer: bool,
//     is_writable: bool,
// }

#[derive(Serialize)]
struct TransferInstructionResponse {
    program_id: String,
    accounts: Vec<AccountMeta>,
    instruction_data: String,
}

#[derive(Serialize)]
struct SimpleTransferInstructionResponse {
    program_id: String,
    accounts: Vec<String>,
    instruction_data: String,
}

#[post("/sol")]
async fn send_sol(req: web::Json<SendSolRequest>) -> impl Responder {
    // Validate inputs
    if req.from.is_empty() || req.to.is_empty() || req.lamports == 0 {
        return HttpResponse::BadRequest().json(ErrorResponse {
            success: false,
            error: "Missing or invalid required fields".to_string(),
        });
    }

    match create_sol_transfer_instruction(&req) {
        Ok(data) => HttpResponse::Ok().json(ApiResponse {
            success: true,
            data,
        }),
        Err(err) => HttpResponse::BadRequest().json(ErrorResponse {
            success: false,
            error: err.to_string(),
        }),
    }
}

#[post("/token")]
async fn send_token(req: web::Json<SendTokenRequest>) -> impl Responder {
    // Validate inputs
    if req.destination.is_empty() || req.mint.is_empty() || req.owner.is_empty() || req.amount == 0
    {
        return HttpResponse::BadRequest().json(ErrorResponse {
            success: false,
            error: "Missing or invalid required fields".to_string(),
        });
    }

    match create_token_transfer_instruction(&req) {
        Ok(data) => HttpResponse::Ok().json(ApiResponse {
            success: true,
            data,
        }),
        Err(err) => HttpResponse::BadRequest().json(ErrorResponse {
            success: false,
            error: err.to_string(),
        }),
    }
}

fn create_sol_transfer_instruction(
    req: &SendSolRequest,
) -> Result<SimpleTransferInstructionResponse> {
    // Parse public keys
    let from_pubkey = Pubkey::from_str(&req.from)
        .map_err(|e| ServerError::InvalidInput(format!("Invalid from address: {}", e)))?;

    let to_pubkey = Pubkey::from_str(&req.to)
        .map_err(|e| ServerError::InvalidInput(format!("Invalid to address: {}", e)))?;

    // Create the transfer instruction
    let instruction = system_instruction::transfer(&from_pubkey, &to_pubkey, req.lamports);

    // Format response
    let accounts = instruction
        .accounts
        .iter()
        .map(|account| account.pubkey.to_string())
        .collect();

    Ok(SimpleTransferInstructionResponse {
        program_id: instruction.program_id.to_string(),
        accounts,
        instruction_data: general_purpose::STANDARD.encode(&instruction.data),
    })
}

fn create_token_transfer_instruction(
    req: &SendTokenRequest,
) -> Result<TransferInstructionResponse> {
    // Parse pubkeys from the request
    let owner = Pubkey::from_str(&req.owner)
        .map_err(|e| ServerError::InvalidInput(format!("Invalid owner address: {}", e)))?;

    let mint = Pubkey::from_str(&req.mint)
        .map_err(|e| ServerError::InvalidInput(format!("Invalid mint address: {}", e)))?;

    let destination_wallet = Pubkey::from_str(&req.destination)
        .map_err(|e| ServerError::InvalidInput(format!("Invalid destination address: {}", e)))?;

    // Get source token account (associated token account for owner)
    let source_token_account = get_associated_token_address(&owner, &mint);

    // Get destination token account (associated token account for destination)
    let destination_token_account = get_associated_token_address(&destination_wallet, &mint);

    // Create the token transfer instruction
    let transfer_instruction = token_instruction::transfer(
        &spl_token::id(),
        &source_token_account,
        &destination_token_account,
        &owner,
        &[], // No additional signers
        req.amount,
    )
    .map_err(|e| {
        ServerError::TokenError(format!(
            "Failed to create token transfer instruction: {}",
            e
        ))
    })?;

    // Extract accounts from the instruction
    let accounts = transfer_instruction
        .accounts
        .iter()
        .map(|acc| AccountMeta {
            pubkey: acc.pubkey,
            is_signer: acc.is_signer,
            is_writable: acc.is_writable,
        })
        .collect();

    // Base64 encode the instruction data
    let instruction_data = general_purpose::STANDARD.encode(&transfer_instruction.data);

    Ok(TransferInstructionResponse {
        program_id: transfer_instruction.program_id.to_string(),
        accounts,
        instruction_data,
    })
}
