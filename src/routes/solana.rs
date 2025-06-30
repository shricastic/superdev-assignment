use actix_web::{post, web, HttpResponse, Result};
use serde::{Deserialize, Serialize};
use solana_sdk::{
    pubkey::Pubkey,
    system_instruction,
};
use base64;
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

#[derive(Deserialize)]
struct SendSolRequest {
    #[serde(alias = "from")]
    sender: String,
    #[serde(alias = "to")]
    recipient: String,
    amount: u64,
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

#[post("/send/sol")]
async fn send_sol(req: web::Json<SendSolRequest>) -> Result<HttpResponse> {
    let sender_pk = match validate_pubkey(&req.sender) {
        Ok(key) => key,
        Err(err) => return Ok(create_error_response(err)),
    };

    let recipient_pk = match validate_pubkey(&req.recipient) {
        Ok(key) => key,
        Err(err) => return Ok(create_error_response(err)),
    };

    if req.amount == 0 {
        return Ok(create_error_response("Amount must be greater than 0"));
    }

    let ix = system_instruction::transfer(&sender_pk, &recipient_pk, req.amount);

    let accounts = ix.accounts.into_iter().map(|m| AccountMetaResponse {
        pubkey: m.pubkey.to_string(),
        is_signer: m.is_signer,
        is_writable: m.is_writable,
    }).collect();

    let data = InstructionData {
        program_id: ix.program_id.to_string(),
        accounts,
        instruction_data: base64::encode(&ix.data),
    };

    Ok(HttpResponse::Ok().json(SuccessResponse { success: true, data }))
}
