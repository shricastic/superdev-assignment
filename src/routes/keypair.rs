use actix_web::{post, HttpResponse, Responder};
use solana_sdk::{
    signature::{Keypair, Signer}, 
};
use bs58;

use crate::util::types::ApiResponse;

#[post("/keypair")]
async fn generate_keypair() -> impl Responder {
    let keypair = Keypair::new();

    let response = serde_json::json!({
        "pubkey": keypair.pubkey().to_string(),
        "secret": bs58::encode(keypair.to_bytes()).into_string(),
    });

    HttpResponse::Ok().json(ApiResponse::Success { success: true, data: response })
}
