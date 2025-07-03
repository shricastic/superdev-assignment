use serde::Serialize;

#[derive(Serialize)]
#[serde(untagged)]
pub enum ApiResponse<T> {
    Success {
        success: bool, 
        data: T,
    },
    Error {
        success: bool, 
        error: String,
    },
}
