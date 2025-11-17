use axum::{extract::Extension, routing::get, Json, Router};

use crate::{auth::jkws::Jwks, http::types::AppError};

/* #[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Jwks {
    pub keys: Vec<Jwk>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Jwk {
    pub alg: String,
    pub e: String,
    pub kid: String,
    pub kty: String,
    pub n: String,
    pub r#use: String,
} */

pub fn jwks_router() -> Router {
    Router::new().route("/jwks", get(get_jwks))
}

async fn get_jwks(
    Extension(context): Extension<crate::http::ApiContext>,
) -> Result<Json<Jwks>, AppError> {
    // Generate JWKS from the AuthService's public key
    let jwks = context
        .auth_service
        .generate_jwks()
        .map_err(|e| AppError::Internal(e.to_string()))?;

    Ok(Json(jwks))
}
