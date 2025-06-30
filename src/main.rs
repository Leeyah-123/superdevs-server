mod error;
mod keypair;
mod message;
mod response;
mod send;
mod token;

use actix_cors::Cors;
use actix_web::{App, HttpResponse, HttpServer, Responder, get};
use dotenv::dotenv;
use std::env;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

use crate::keypair::routes as keypair_routes;
use crate::message::routes as message_routes;
use crate::response::ApiResponse;
use crate::send::routes as send_routes;
use crate::token::routes as token_routes;

#[get("/health")]
async fn health_check() -> impl Responder {
    HttpResponse::Ok().json(ApiResponse {
        success: true,
        data: "Solana HTTP Server is running".to_string(),
    })
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    // Load environment variables
    dotenv().ok();

    // Initialize logger
    tracing_subscriber::registry()
        .with(tracing_subscriber::EnvFilter::new(
            std::env::var("RUST_LOG").unwrap_or_else(|_| "info".into()),
        ))
        .with(tracing_subscriber::fmt::layer())
        .init();

    let port = env::var("PORT")
        .unwrap_or_else(|_| "8080".to_string())
        .parse::<u16>()
        .unwrap();
    let bind_address = env::var("BIND_ADDRESS").unwrap_or_else(|_| "127.0.0.1".to_string());

    tracing::info!("Starting server at http://{}:{}", bind_address, port);

    HttpServer::new(|| {
        // Configure CORS
        let cors = Cors::default()
            .allow_any_origin()
            .allow_any_method()
            .allow_any_header()
            .max_age(3600);

        // Register our app routes, services and middleware
        App::new()
            .wrap(cors)
            .configure(keypair_routes::configure)
            .configure(token_routes::configure)
            .configure(message_routes::configure)
            .configure(send_routes::configure)
    })
    .bind((bind_address, port))?
    .run()
    .await
}
