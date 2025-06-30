use actix_web::{HttpResponse, Responder, post, web};
use base64::{Engine as _, engine::general_purpose};
use bs58;
use serde::{Deserialize, Serialize};
use solana_sdk::{
    pubkey::Pubkey,
    signature::{Keypair, Signature, Signer},
};

use crate::error::{Result, ServerError};
use crate::response::{ApiResponse, ErrorResponse};

pub mod routes {
    use super::*;
    use actix_web::web;

    pub fn configure(cfg: &mut web::ServiceConfig) {
        cfg.service(web::scope("/message").service(sign).service(verify));
    }
}

#[derive(Deserialize)]
pub struct SignMessageRequest {
    message: String,
    secret: String,
}

#[derive(Serialize)]
pub struct SignMessageData {
    signature: String,
    public_key: String,
    message: String,
}

#[derive(Deserialize)]
pub struct VerifyMessageRequest {
    message: String,
    pubkey: String,
    signature: String,
}

#[derive(Serialize)]
pub struct VerifyMessageData {
    valid: bool,
    message: String,
    pubkey: String,
}

#[post("/sign")]
async fn sign(req: web::Json<SignMessageRequest>) -> impl Responder {
    match sign_message(&req.message, &req.secret) {
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

#[post("/verify")]
async fn verify(req: web::Json<VerifyMessageRequest>) -> impl Responder {
    match verify_message(&req.message, &req.pubkey, &req.signature) {
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

fn sign_message(message: &str, secret: &str) -> Result<SignMessageData> {
    if message.is_empty() || secret.is_empty() {
        return Err(ServerError::InvalidInput(
            "Missing required fields".to_string(),
        ));
    }

    // Decode the keypair from base58
    let keypair_bytes = bs58::decode(secret)
        .into_vec()
        .map_err(|e| ServerError::Base58DecodeError(e))?;

    if keypair_bytes.len() != 64 {
        return Err(ServerError::InvalidInput(
            "Invalid secret key length".to_string(),
        ));
    }

    // Create keypair from bytes
    let keypair = Keypair::from_bytes(&keypair_bytes)
        .map_err(|e| ServerError::InvalidInput(format!("Failed to create keypair: {}", e)))?;

    // Sign the message
    let signature = keypair.sign_message(message.as_bytes());
    let public_key = keypair.pubkey().to_string();

    Ok(SignMessageData {
        message: message.to_string(),
        public_key,
        signature: general_purpose::STANDARD.encode(signature.as_ref()),
    })
}

fn verify_message(
    message: &str,
    pubkey_str: &str,
    signature_str: &str,
) -> Result<VerifyMessageData> {
    if message.is_empty() || pubkey_str.is_empty() || signature_str.is_empty() {
        return Err(ServerError::InvalidInput(
            "Missing required fields".to_string(),
        ));
    }

    // Parse the pubkey
    let pubkey = Pubkey::try_from(pubkey_str)
        .map_err(|e| ServerError::InvalidInput(format!("Invalid pubkey: {}", e)))?;

    // Assume base64 encoded signature as per spec
    let signature_bytes = general_purpose::STANDARD
        .decode(signature_str)
        .map_err(|e| ServerError::InvalidInput(format!("Invalid signature format: {}", e)))?;

    if signature_bytes.len() != 64 {
        return Err(ServerError::InvalidInput(
            "Invalid signature length".to_string(),
        ));
    }

    // Convert bytes to signature
    let signature = Signature::try_from(signature_bytes.as_slice())
        .map_err(|e| ServerError::InvalidInput(format!("Invalid signature: {}", e)))?;

    // Verify the message
    let valid = signature.verify(pubkey.as_ref(), message.as_bytes());

    Ok(VerifyMessageData {
        valid,
        message: message.to_string(),
        pubkey: pubkey_str.to_string(),
    })
}
