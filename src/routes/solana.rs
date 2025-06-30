use std::str::FromStr;

use actix_web::{post, web, HttpResponse, Responder};
use serde::{Serialize, Deserialize};
use solana_sdk::{pubkey::Pubkey, system_instruction};

use crate::util::types::ApiResponse;

#[derive(Deserialize)]
pub struct SolTransferReq {
    pub from: String,
    pub to: String,
    pub lamports: u64,
}

#[derive(Serialize)]
pub struct SolTransferRes {
    pub program_id: String,
    pub accounts: Vec<String>,
    instruction_data: String,
}

#[post("/send/sol")]
pub async fn send_sol(req: web::Json<SolTransferReq>) -> impl Responder {
    let from_pubkey = match Pubkey::from_str(&req.from) {
        Ok(pk) => pk,
        Err(_) => return HttpResponse::BadRequest().json(ApiResponse::<()>::Error { success: false, error: "sender pubkey invalid".into(), })
    };


    let to_pubkey = match Pubkey::from_str(&req.to) {
        Ok(pk) => pk,
        Err(_) => return HttpResponse::BadRequest().json(ApiResponse::<()>::Error { success: false, error: "receiver Pubkey invalid".into(), }) 
    };

    if req.lamports == 0 {
        return HttpResponse::BadRequest().json(ApiResponse::<()>::Error { success: false, error: "amout should be greater than 0".into() })
    }

    let make_instruction = system_instruction::transfer(&from_pubkey, &to_pubkey, req.lamports);

    let res = SolTransferRes {
        program_id: make_instruction.program_id.to_string(),
        accounts: make_instruction 
            .accounts
            .iter()
            .map(|acc| acc.pubkey.to_string())
            .collect(),
        instruction_data: bs58::encode(&make_instruction.data).into_string(),
    };

    HttpResponse::Ok().json(ApiResponse::Success { success: true, data: res })
}
