use serde::Serialize;

/// Standard successful response wrapper for all endpoints
#[derive(Serialize)]
pub struct ApiResponse<T> {
    pub success: bool,
    pub data: T,
}

/// Standard error response for all endpoints
#[derive(Serialize)]
pub struct ErrorResponse {
    pub success: bool,
    pub error: String,
}
