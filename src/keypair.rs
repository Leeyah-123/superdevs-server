use actix_web::{HttpResponse, Responder, post};
use bs58;
use serde::Serialize;
use solana_sdk::signature::{Keypair, Signer};

use crate::error::Result;
use crate::response::{ApiResponse, ErrorResponse};

pub mod routes {
    use super::*;
    use actix_web::web;

    pub fn configure(cfg: &mut web::ServiceConfig) {
        cfg.service(web::scope("").service(generate_keypair));
    }
}

#[derive(Serialize)]
struct KeypairResponse {
    pubkey: String,
    secret: String,
}

#[post("/keypair")]
async fn generate_keypair() -> impl Responder {
    match create_new_keypair() {
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

fn create_new_keypair() -> Result<KeypairResponse> {
    let keypair = Keypair::new();
    let pubkey = keypair.pubkey().to_string();
    let secret = bs58::encode(keypair.to_bytes()).into_string();

    Ok(KeypairResponse { pubkey, secret })
}
