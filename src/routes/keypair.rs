use actix_web::{post, HttpResponse, Responder};
use serde::{Serialize};
use solana_sdk::{
    signature::{Keypair, Signer}, 
};
use bs58;

#[derive(Serialize)]
struct KeypairData {
    pubkey: String,
    secret: String,
}

#[derive(Serialize)]
struct SuccessResponse<T> {
    success: bool,
    data: T,
}

#[post("/keypair")]
async fn generate_keypair() -> impl Responder {
    let keypair = Keypair::new();
    let pubkey = keypair.pubkey().to_string();
    let secret_bytes = keypair.to_bytes(); 
    let secret_base58 = bs58::encode(secret_bytes).into_string();
    
    let response = SuccessResponse {
        success: true,
        data: KeypairData {
            pubkey,
            secret: secret_base58,
        },
    };
    
    HttpResponse::Ok().json(response)
}
