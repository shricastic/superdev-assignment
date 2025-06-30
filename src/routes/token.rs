use actix_web::{post, web, HttpResponse, Responder, Result};
use serde::{Deserialize, Serialize};
use solana_sdk::{
    signature::{Keypair, Signature, Signer},
    pubkey::Pubkey,
};
use spl_token::instruction::{initialize_mint, mint_to, transfer};
use spl_associated_token_account::get_associated_token_address;
use base64::{Engine as _, engine::general_purpose};
use bs58;
use std::str::FromStr;

#[derive(Serialize)]
struct SuccessResponse<T> {
    success: bool,
    data: T,
}

#[derive(Serialize)]
struct ErrorResponse {
    success: bool,
    error: String,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct AccountMetaResponse {
    pubkey: String,
    is_signer: bool,
    is_writable: bool,
}

#[derive(Serialize)]
struct InstructionData {
    program_id: String,
    accounts: Vec<AccountMetaResponse>,
    instruction_data: String,
}

fn create_error_response<E: Into<String>>(err: E) -> HttpResponse {
    HttpResponse::BadRequest().json(ErrorResponse {
        success: false,
        error: err.into(),
    })
}

fn validate_pubkey(key_str: &str) -> Result<Pubkey, String> {
    Pubkey::from_str(key_str).map_err(|_| format!("Invalid pubkey: {}", key_str))
}

#[derive(Deserialize)]
struct SignMessageRequest {
    message: String,
    secret: String,
}

#[derive(Serialize)]
struct SignMessageData {
    signature: String,
    public_key: String,
    message: String,
}

#[post("/message/sign")]
async fn sign_message(req: web::Json<SignMessageRequest>) -> impl Responder {
    if req.message.is_empty() || req.secret.is_empty() {
        return create_error_response("Missing required fields");
    }

    let secret_bytes = match bs58::decode(&req.secret).into_vec() {
        Ok(bytes) if bytes.len() == 64 => bytes,
        _ => return create_error_response("Invalid secret key"),
    };

    let keypair = match Keypair::from_bytes(&secret_bytes) {
        Ok(kp) => kp,
        Err(_) => return create_error_response("Failed to parse keypair"),
    };

    let sig = keypair.sign_message(req.message.as_bytes());
    let data = SignMessageData {
        signature: general_purpose::STANDARD.encode(sig.as_ref()),
        public_key: keypair.pubkey().to_string(),
        message: req.message.clone(),
    };

    HttpResponse::Ok().json(SuccessResponse { success: true, data })
}

#[derive(Deserialize)]
struct VerifyMessageRequest {
    message: String,
    signature: String,
    #[serde(alias = "public_key", alias = "publicKey")]
    pubkey: String,
}

#[derive(Serialize)]
struct VerifyMessageData {
    valid: bool,
    message: String,
    pubkey: String,
}

#[post("/message/verify")]
async fn verify_message(req: web::Json<VerifyMessageRequest>) -> impl Responder {
    let pk = match Pubkey::from_str(&req.pubkey) {
        Ok(v) => v,
        Err(_) => return create_error_response("Invalid public key"),
    };

    let sig_bytes = match general_purpose::STANDARD.decode(&req.signature) {
        Ok(bytes) => bytes,
        Err(_) => return create_error_response("Invalid signature encoding"),
    };

    let sig = match Signature::try_from(sig_bytes.as_slice()) {
        Ok(s) => s,
        Err(_) => return create_error_response("Invalid signature format"),
    };

    let valid = sig.verify(pk.as_ref(), req.message.as_bytes());
    let data = VerifyMessageData {
        valid,
        message: req.message.clone(),
        pubkey: req.pubkey.clone(),
    };

    HttpResponse::Ok().json(SuccessResponse { success: true, data })
}

#[derive(Deserialize)]
struct CreateTokenRequest {
    #[serde(rename = "mintAuthority")]
    mint_authority: String,
    mint: String,
    decimals: u8,
}

#[post("/token/create")]
async fn create_token(req: web::Json<CreateTokenRequest>) -> Result<HttpResponse> {
    let mint_pk = match validate_pubkey(&req.mint) {
        Ok(key) => key,
        Err(err) => return Ok(create_error_response(err)),
    };

    let auth_pk = match validate_pubkey(&req.mint_authority) {
        Ok(key) => key,
        Err(err) => return Ok(create_error_response(err)),
    };

    if req.decimals > 9 {
        return Ok(create_error_response("Decimals must be between 0 and 9"));
    }

    let ix = match initialize_mint(
        &spl_token::id(),
        &mint_pk,
        &auth_pk,
        None,
        req.decimals,
    ) {
        Ok(ix) => ix,
        Err(e) => return Ok(create_error_response(format!("Init mint failed: {}", e))),
    };

    let accounts = ix.accounts.into_iter().map(|m| AccountMetaResponse {
        pubkey: m.pubkey.to_string(),
        is_signer: m.is_signer,
        is_writable: m.is_writable,
    }).collect();

    let data = InstructionData {
        program_id: spl_token::id().to_string(),
        accounts,
        instruction_data: general_purpose::STANDARD.encode(&ix.data),
    };

    Ok(HttpResponse::Ok().json(SuccessResponse { success: true, data }))
}

#[derive(Deserialize)]
struct MintTokenRequest {
    mint: String,
    destination: String,
    authority: String,
    amount: u64,
}

#[post("/token/mint")]
async fn mint_token(req: web::Json<MintTokenRequest>) -> Result<HttpResponse> {
    let mint_pk = match validate_pubkey(&req.mint) {
        Ok(key) => key,
        Err(err) => return Ok(create_error_response(err)),
    };

    let dst_pk = match validate_pubkey(&req.destination) {
        Ok(key) => key,
        Err(err) => return Ok(create_error_response(err)),
    };

    let auth_pk = match validate_pubkey(&req.authority) {
        Ok(key) => key,
        Err(err) => return Ok(create_error_response(err)),
    };

    if req.amount == 0 {
        return Ok(create_error_response("Amount must be greater than 0"));
    }

    let ix = match mint_to(
        &spl_token::id(),
        &mint_pk,
        &dst_pk,
        &auth_pk,
        &[],
        req.amount,
    ) {
        Ok(ix) => ix,
        Err(e) => return Ok(create_error_response(format!("MintTo failed: {}", e))),
    };

    let accounts = ix.accounts.into_iter().map(|m| AccountMetaResponse {
        pubkey: m.pubkey.to_string(),
        is_signer: m.is_signer,
        is_writable: m.is_writable,
    }).collect();

    let data = InstructionData {
        program_id: ix.program_id.to_string(),
        accounts,
        instruction_data: general_purpose::STANDARD.encode(&ix.data),
    };

    Ok(HttpResponse::Ok().json(SuccessResponse { success: true, data }))
}

#[derive(Deserialize)]
struct SendTokenRequest {
    destination: String,
    mint: String,
    owner: String,
    amount: u64,
}

#[post("/send/token")]
async fn send_token(req: web::Json<SendTokenRequest>) -> Result<HttpResponse> {
    let dst_pk = match validate_pubkey(&req.destination) {
        Ok(key) => key,
        Err(err) => return Ok(create_error_response(err)),
    };

    let mint_pk = match validate_pubkey(&req.mint) {
        Ok(key) => key,
        Err(err) => return Ok(create_error_response(err)),
    };

    let owner_pk = match validate_pubkey(&req.owner) {
        Ok(key) => key,
        Err(err) => return Ok(create_error_response(err)),
    };

    if req.amount == 0 {
        return Ok(create_error_response("Amount must be greater than 0"));
    }

    let src_ata = get_associated_token_address(&owner_pk, &mint_pk);
    let dst_ata = get_associated_token_address(&dst_pk, &mint_pk);

    let ix = match transfer(
        &spl_token::id(),
        &src_ata,
        &dst_ata,
        &owner_pk,
        &[],
        req.amount,
    ) {
        Ok(ix) => ix,
        Err(e) => return Ok(create_error_response(format!("Transfer failed: {}", e))),
    };

    let accounts = ix.accounts.into_iter().map(|m| AccountMetaResponse {
        pubkey: m.pubkey.to_string(),
        is_signer: m.is_signer,
        is_writable: m.is_writable,
    }).collect();

    let data = InstructionData {
        program_id: ix.program_id.to_string(),
        accounts,
        instruction_data: general_purpose::STANDARD.encode(&ix.data),
    };

    Ok(HttpResponse::Ok().json(SuccessResponse { success: true, data }))
}
