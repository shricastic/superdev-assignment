use actix_web::{post, web, HttpResponse, Responder};
use base64::{engine::general_purpose, Engine};
use serde::{Deserialize, Serialize};
use solana_sdk::{ pubkey::Pubkey };
use spl_associated_token_account::get_associated_token_address;
use spl_token::instruction::{initialize_mint, mint_to, transfer};
use std::str::FromStr;

use crate::util::types::ApiResponse;

#[derive(Deserialize)]
struct TokenCreateReq {
    mintAuthority: String,
    mint: String,
    decimals: u8
}

#[derive(Serialize)]
struct TokenRes {
    program_id: String,
    accounts: Vec<TokenCreateResAcc>,
    instruction_data: String
}

#[derive(Serialize)]
struct TokenCreateResAcc {
    pubkey: String,
    is_signer: bool,
    is_writable: bool
}

#[post("/token/create")]
async fn create_token(req: web::Json<TokenCreateReq>) -> impl Responder {
    if req.mintAuthority.trim().is_empty() || req.mint.trim().is_empty() {
        return HttpResponse::BadRequest().json(ApiResponse::<()>::Error { success: false, error: "missing inputs".into(),});
    }

    if req.decimals > 9 {
        return HttpResponse::BadRequest().json(ApiResponse::<()>::Error {success: false, error: "invalid decimal places".into() });
    }

    let mint_pubkey = match Pubkey::from_str(&req.mint) {
        Ok(pk) => pk,
        Err(_) => return HttpResponse::BadRequest().json(ApiResponse::<()>::Error { success: false, error: "invalid mint key".into(), })
    };

    let mint_auth_pubkey = match Pubkey::from_str(&req.mintAuthority) {
        Ok(pk) => pk,
        Err(_) => return HttpResponse::BadRequest().json(ApiResponse::<()>::Error { success: false, error: "invalid mintAuthority public key".into() }),
    };

    let ix = match initialize_mint(
        &spl_token::id(),
        &mint_pubkey,
        &mint_auth_pubkey,
        None,
        req.decimals,
    ) {
        Ok(ix) => ix,
        Err(e) => return HttpResponse::BadRequest().json(ApiResponse::<()>::Error {
            success: false,
            error: format!("Init mint failed: {}", e),
        }),
    };

    let accounts: Vec<TokenCreateResAcc> = ix
        .accounts
        .iter()
        .map(|acc| TokenCreateResAcc {
            pubkey: acc.pubkey.to_string(),
            is_signer: acc.is_signer,
            is_writable: acc.is_writable,
        })
        .collect();

    let response = TokenRes {
        program_id: ix.program_id.to_string(),
        accounts,
        instruction_data: base64::encode(&ix.data),
    };

    HttpResponse::Ok().json(ApiResponse::Success {
        success: true,
        data: response,
    })
}

#[derive(Deserialize)]
struct MintTokenReq {
    mint: String,
    destination: String,
    authority: String,
    amount: u64
}

#[post("/token/mint")]
pub async fn mint_token(req: web::Json<MintTokenReq>) -> impl Responder {
    if req.amount <= 0 {
        return HttpResponse::BadRequest().json(ApiResponse::<()>::Error {success: false, error: "invalid amount".into() });
    }

    let mint_pubkey = match Pubkey::from_str(&req.mint) {
        Ok(pk) => pk,
        Err(_) => return HttpResponse::BadRequest().json(ApiResponse::<()>::Error { success: false, error: "invalid mint key".into(), })
    };


    let dest_pubkey = match Pubkey::from_str(&req.destination) {
        Ok(pk) => pk,
        Err(_) => return HttpResponse::BadRequest().json(ApiResponse::<()>::Error { success: false, error: "invalid destination key".into(), })
    };

    let mint_auth_pubkey = match Pubkey::from_str(&req.authority) {
        Ok(pk) => pk,
        Err(_) => return HttpResponse::BadRequest().json(ApiResponse::<()>::Error { success: false, error: "invalid authority key".into() }),
    };

    let ata = get_associated_token_address(&dest_pubkey, &mint_pubkey);

    let instruction = match mint_to( &spl_token::id(), &mint_pubkey, &ata, &mint_auth_pubkey, &[], req.amount) {
        Ok(instruction) => instruction,
        Err(e) => return HttpResponse::BadRequest().json(ApiResponse::<()>::Error {success: false, error: format!("Failed to create instruction: {}", e) })
    };

    let accounts = instruction
        .accounts
        .iter()
        .map(|acc| TokenCreateResAcc {
            pubkey: acc.pubkey.to_string(),
            is_signer: acc.is_signer,
            is_writable: acc.is_writable,
        })
        .collect();

    let res = TokenRes {
        program_id: instruction.program_id.to_string(),
        accounts,
        instruction_data: general_purpose::STANDARD.encode(&instruction.data),
    };

    HttpResponse::Ok().json(ApiResponse::Success { success: true, data: res })
}

#[derive(Deserialize)]
struct SendTokenReq {
    mint: String,
    destination: String,
    owner: String,
    amount: u64
}

#[derive(Serialize)]
struct TokenSendRes {
    program_id: String,
    accounts: Vec<TokenSendResAcc>,
    instruction_data: String
}

#[derive(Serialize)]
struct TokenSendResAcc {
    pubkey: String,
    isSigner: bool,
}

#[post("/send/token")]
pub async fn send_token(req: web::Json<SendTokenReq>) -> impl Responder {
    if req.amount == 0 {
        return HttpResponse::BadRequest().json(ApiResponse::<()>::Error {success: false, error: "invalid amount".into() });
    }

    let mint_pubkey = match Pubkey::from_str(&req.mint) {
        Ok(pk) => pk,
        Err(_) => return HttpResponse::BadRequest().json(ApiResponse::<()>::Error { success: false, error: "invalid mint key".into(), })
    };


    let dest_pubkey = match Pubkey::from_str(&req.destination) {
        Ok(pk) => pk,
        Err(_) => return HttpResponse::BadRequest().json(ApiResponse::<()>::Error { success: false, error: "invalid destination key".into(), })
    };

    let mint_auth_pubkey = match Pubkey::from_str(&req.owner) {
        Ok(pk) => pk,
        Err(_) => return HttpResponse::BadRequest().json(ApiResponse::<()>::Error { success: false, error: "invalid authority key".into() }),
    };

    let source_ata = get_associated_token_address(&mint_auth_pubkey, &mint_pubkey);
    let destination_ata = get_associated_token_address(&dest_pubkey, &mint_pubkey);

    let instruction = match transfer( &spl_token::id(), &source_ata, &destination_ata, &mint_auth_pubkey, &[], req.amount) {
        Ok(ix) => ix,
        Err(e) => return HttpResponse::BadRequest().json(ApiResponse::<()>::Error { success: false, error: format!("Failed to create transfer instruction: {}", e)})
    };

    let accounts = vec![
        TokenSendResAcc {
            pubkey: mint_auth_pubkey.to_string(),
            isSigner: false,
        },
        TokenSendResAcc {
            pubkey: destination_ata.to_string(),
            isSigner: false,
        },
        TokenSendResAcc {
            pubkey: mint_auth_pubkey.to_string(),
            isSigner: false,
        },
    ];    let res = TokenSendRes {
        program_id: instruction.program_id.to_string(),
        accounts,
        instruction_data: general_purpose::STANDARD.encode(&instruction.data),
    };

    HttpResponse::Ok().json(ApiResponse::Success { success: true, data: res })
}
