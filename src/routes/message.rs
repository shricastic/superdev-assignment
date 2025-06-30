use actix_web::{post, web, HttpResponse, Responder};
use serde::{Deserialize, Serialize};
use solana_sdk::{signature::{Keypair, Signature, Signer}, pubkey::Pubkey};
use base64;
use bs58;
use std::str::FromStr;

#[derive(Deserialize)]
struct SignMessageRequest {
    message: String,
    secret:  String,
}

#[derive(Serialize)]
struct SignMessageResponse {
    success: bool,
    data:    Option<SignMessageData>,
    error:   Option<String>,
}

#[derive(Serialize)]
struct SignMessageData {
    signature:  String,
    public_key: String,
    message:    String,
}

#[post("/message/sign")]
async fn sign_message(req: web::Json<SignMessageRequest>) -> impl Responder {
    if req.message.is_empty() || req.secret.is_empty() {
        return HttpResponse::BadRequest().json(SignMessageResponse {
            success: false,
            data:    None,
            error:   Some("Missing required fields".into()),
        });
    }

    let secret_bytes = match bs58::decode(&req.secret).into_vec() {
        Ok(bytes) if bytes.len() == 64 => bytes,
        _ => {
            return HttpResponse::BadRequest().json(SignMessageResponse {
                success: false,
                data:    None,
                error:   Some("Invalid secret key".into()),
            });
        }
    };

    let keypair = match Keypair::from_bytes(&secret_bytes) {
        Ok(kp) => kp,
        Err(_) => {
            return HttpResponse::BadRequest().json(SignMessageResponse {
                success: false,
                data:    None,
                error:   Some("Failed to parse keypair".into()),
            });
        }
    };

    let signature: Signature = keypair.sign_message(req.message.as_bytes());

    HttpResponse::Ok().json(SignMessageResponse {
        success: true,
        data: Some(SignMessageData {
            signature:  base64::encode(signature.as_ref()),
            public_key: keypair.pubkey().to_string(),
            message:    req.message.clone(),
        }),
        error: None,
    })
}

#[derive(Deserialize)]
struct VerifyMessageRequest {
    message:   String,
    signature: String,

    /// Accept either `"pubkey"` or `"public_key"` in the incoming JSON
    #[serde(alias = "public_key")]
    pubkey:    String,
}

#[derive(Serialize)]
struct VerifyMessageResponse {
    success: bool,
    data:    Option<VerifyMessageData>,
    error:   Option<String>,
}

#[derive(Serialize)]
struct VerifyMessageData {
    valid:   bool,
    message: String,
    pubkey:  String,
}

#[post("/message/verify")]
async fn verify_message(req: web::Json<VerifyMessageRequest>) -> impl Responder {
    let pubkey = match Pubkey::from_str(&req.pubkey) {
        Ok(pk) => pk,
        Err(_) => {
            return HttpResponse::BadRequest().json(VerifyMessageResponse {
                success: false,
                data:    None,
                error:   Some("Invalid public key".into()),
            });
        }
    };

    let signature = match base64::decode(&req.signature) {
        Ok(sig_bytes) => match Signature::try_from(sig_bytes.as_slice()) {
            Ok(sig) => sig,
            Err(_) => {
                return HttpResponse::BadRequest().json(VerifyMessageResponse {
                    success: false,
                    data:    None,
                    error:   Some("Invalid signature format".into()),
                });
            }
        },
        Err(_) => {
            return HttpResponse::BadRequest().json(VerifyMessageResponse {
                success: false,
                data:    None,
                error:   Some("Signature not base64".into()),
            });
        }
    };

    let is_valid = signature.verify(pubkey.as_ref(), req.message.as_bytes());

    HttpResponse::Ok().json(VerifyMessageResponse {
        success: true,
        data: Some(VerifyMessageData {
            valid:   is_valid,
            message: req.message.clone(),
            pubkey:  req.pubkey.clone(),
        }),
        error: None,
    })
}
