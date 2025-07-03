use actix_web::{post, web, HttpResponse, Responder};
use serde::{Deserialize, Serialize};
use solana_sdk::{signature::{Keypair, Signature, Signer}, pubkey::Pubkey};
use bs58;
use std::str::FromStr;

use crate::util::types::ApiResponse;

#[derive(Deserialize)]
struct SignMessageReq {
    message: String,
    secret:  String,
}

#[derive(Serialize)]
struct SignMessageRes {
    signature:  String,
    pubkey: String,
    message:    String,
}

#[post("/message/sign")]
async fn sign_message(req: web::Json<SignMessageReq>) -> impl Responder {
    if req.message.is_empty() || req.secret.is_empty() {
        return HttpResponse::BadRequest().json(ApiResponse::<()>::Error { success: false, error: "message or secret field is empty".into() });
    }

    let secret_bytes = match bs58::decode(&req.secret).into_vec() {
        Ok(bytes) if bytes.len() == 64 => bytes,
        _ => return HttpResponse::BadRequest().json(ApiResponse::<()>::Error {
            success: false,
            error: "Invalid secret key".into(),
        }),
    };

    let keypair = match Keypair::from_bytes(&secret_bytes) {
        Ok(kp) => kp,
        Err(_) => return HttpResponse::BadRequest().json(ApiResponse::<()>::Error { success: false, error: "Failed to parse keypair".into() }),
    };    

    let signature: Signature = keypair.sign_message(req.message.as_bytes());

    let res = SignMessageRes {
        signature:  signature.to_string(),
        pubkey: keypair.pubkey().to_string(),
        message:    req.message.clone(),
    };

    HttpResponse::Ok().json(ApiResponse::Success { success: true, data: res })
}

#[derive(Deserialize)]
struct VerifyMessageReq {
    message:   String,
    signature: String,
    pubkey:    String,
}

#[derive(Serialize)]
struct VerifyMessageRes {
    valid:   bool,
    message: String,
    pubkey:  String,
}

#[post("/message/verify")]
async fn verify_message(req: web::Json<VerifyMessageReq>) -> impl Responder {
    let pubkey = match Pubkey::from_str(&req.pubkey) {
        Ok(pk) => pk,
        Err(_) => return HttpResponse::BadRequest().json(ApiResponse::<()>::Error { success: false, error: "pubkey invalid".into() })
    };

    let signature = match bs58::decode(&req.signature).into_vec() {
        Ok(sig_bytes) => match Signature::try_from(sig_bytes.as_slice()) {
            Ok(sig) => sig,
            Err(_) => return HttpResponse::BadRequest().json(ApiResponse::<()>::Error { success: false, error: "Signature invalid".into() })
        },
        Err(_) => return HttpResponse::BadRequest().json(ApiResponse::<()>::Error { success: false, error: "Signature invalid".into() })
    };

    let is_valid = signature.verify(pubkey.as_ref(), req.message.as_bytes());
    let res = VerifyMessageRes{
            valid:   is_valid,
            message: req.message.clone(),
            pubkey:  req.pubkey.clone(),
    };

    HttpResponse::Ok().json(ApiResponse::Success { success: true, data: res })
}
