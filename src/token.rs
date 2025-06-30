use crate::error::{Result, ServerError};
use actix_web::{HttpResponse, Responder, post, web};
use base64::{Engine as _, engine::general_purpose};
use serde::{Deserialize, Serialize};
use solana_sdk::pubkey::Pubkey;
use spl_token::instruction as token_instruction;
use std::str::FromStr;

use crate::response::{ApiResponse, ErrorResponse};

pub mod routes {
    use super::*;
    use actix_web::web;

    pub fn configure(cfg: &mut web::ServiceConfig) {
        cfg.service(
            web::scope("/token")
                .service(create_token)
                .service(mint_token),
        );
    }
}

#[derive(Deserialize)]
struct CreateTokenRequest {
    #[serde(rename = "mintAuthority")]
    mint_authority: String,
    mint: String,
    decimals: u8,
}

#[derive(Deserialize)]
struct MintTokenRequest {
    mint: String,
    destination: String,
    authority: String,
    amount: u64,
}

#[derive(Serialize)]
struct TokenAccountMeta {
    pubkey: String,
    is_signer: bool,
    is_writable: bool,
}

#[derive(Serialize)]
struct CreateTokenResponse {
    program_id: String,
    accounts: Vec<TokenAccountMeta>,
    instruction_data: String,
}

#[post("/create")]
async fn create_token(req: web::Json<CreateTokenRequest>) -> impl Responder {
    match create_token_mint_instruction(&req) {
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

#[post("/mint")]
async fn mint_token(req: web::Json<MintTokenRequest>) -> impl Responder {
    match create_mint_to_instruction(&req) {
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

fn create_token_mint_instruction(req: &CreateTokenRequest) -> Result<CreateTokenResponse> {
    // Parse mint and mint authority pubkeys
    let mint_pubkey = Pubkey::from_str(&req.mint)
        .map_err(|e| ServerError::InvalidInput(format!("Invalid mint pubkey: {}", e)))?;

    let mint_authority = Pubkey::from_str(&req.mint_authority)
        .map_err(|e| ServerError::InvalidInput(format!("Invalid mint authority pubkey: {}", e)))?;

    // Create the token mint instruction
    let token_mint_instruction = token_instruction::initialize_mint(
        &spl_token::id(),
        &mint_pubkey,
        &mint_authority,
        None, // No freeze authority
        req.decimals,
    )
    .map_err(|e| ServerError::TokenError(format!("Failed to create mint instruction: {}", e)))?;

    // Extract accounts from the instruction
    let accounts = token_mint_instruction
        .accounts
        .iter()
        .map(|acc| TokenAccountMeta {
            pubkey: acc.pubkey.to_string(),
            is_signer: acc.is_signer,
            is_writable: acc.is_writable,
        })
        .collect();

    // Base64 encode the instruction data
    let instruction_data = general_purpose::STANDARD.encode(&token_mint_instruction.data);

    Ok(CreateTokenResponse {
        program_id: token_mint_instruction.program_id.to_string(),
        accounts,
        instruction_data,
    })
}

fn create_mint_to_instruction(req: &MintTokenRequest) -> Result<CreateTokenResponse> {
    // Parse pubkeys from the request
    let mint = Pubkey::from_str(&req.mint)
        .map_err(|e| ServerError::InvalidInput(format!("Invalid mint address: {}", e)))?;

    let destination = Pubkey::from_str(&req.destination)
        .map_err(|e| ServerError::InvalidInput(format!("Invalid destination address: {}", e)))?;

    let authority = Pubkey::from_str(&req.authority)
        .map_err(|e| ServerError::InvalidInput(format!("Invalid authority address: {}", e)))?;

    // Create the mint-to instruction
    let mint_to_instruction = token_instruction::mint_to(
        &spl_token::id(),
        &mint,
        &destination,
        &authority,
        &[], // No additional signers
        req.amount,
    )
    .map_err(|e| ServerError::TokenError(format!("Failed to create mint-to instruction: {}", e)))?;

    // Extract accounts from the instruction
    let accounts = mint_to_instruction
        .accounts
        .iter()
        .map(|acc| TokenAccountMeta {
            pubkey: acc.pubkey.to_string(),
            is_signer: acc.is_signer,
            is_writable: acc.is_writable,
        })
        .collect();

    // Base64 encode the instruction data
    let instruction_data = general_purpose::STANDARD.encode(&mint_to_instruction.data);

    Ok(CreateTokenResponse {
        program_id: mint_to_instruction.program_id.to_string(),
        accounts,
        instruction_data,
    })
}
