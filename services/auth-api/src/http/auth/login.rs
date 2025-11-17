use axum::{extract::Extension, routing::post, Json, Router};
use sha2::{Digest, Sha256};

use crate::auth::jkws::Scope;
use crate::http::types::{ApiResponse, AppError, LoginRequest, LoginResponseData, User, UserInfo};
use crate::http::ApiContext;
use crate::misc::validator;

pub fn login_router() -> Router {
    Router::new().route("/login", post(handle_login))
}

// Helper function to determine scopes based on user role
fn determine_user_scopes(role: &str) -> Vec<Scope> {
    match role {
        "admin" => vec![Scope::Viewer, Scope::Manager, Scope::Admin],
        "manager" => vec![Scope::Viewer, Scope::Manager],
        "viewer" => vec![Scope::Viewer],
        _ => vec![Scope::Viewer], // Default to viewer (read-only)
    }
}

// Login handler
async fn handle_login(
    Extension(context): Extension<ApiContext>,
    Json(login_req): Json<LoginRequest>,
) -> Result<Json<ApiResponse<LoginResponseData>>, AppError> {
    // Validate email format
    validator::validate_email(&login_req.email)?;

    // Query the database for the user
    let user = sqlx::query_as::<_, User>(
        "SELECT id, merchant_id, email, password_hash, display_name, role, is_active 
         FROM users WHERE email = $1",
    )
    .bind(&login_req.email)
    .fetch_optional(&context.db)
    .await?;

    let user = user.ok_or(AppError::InvalidCredentials)?;

    // Check if user is active
    if !user.is_active {
        return Err(AppError::Unauthorized);
    }

    // Verify password
    let password_hash = user.password_hash.ok_or(AppError::InvalidCredentials)?;
    
    // Hash the provided password
    let mut hasher = Sha256::new();
    hasher.update(login_req.password.as_bytes());
    let provided_hash = format!("{:x}", hasher.finalize());

    if password_hash != provided_hash {
        println!("Password mismatch for user: {}", user.email);
        return Err(AppError::InvalidCredentials);
    }

    // Determine scopes based on user's role
    let scopes = determine_user_scopes(&user.role);

    // Generate JWT token pair (access + refresh)
    let (access_token, refresh_token) = context
        .auth_service
        .gen_token_pair(user.id, user.email.clone(), scopes)
        .map_err(|_| AppError::InternalServerError)?;

    let response_data = LoginResponseData {
        access_token,
        refresh_token,
        user: UserInfo {
            id: user.id,
            email: user.email,
            display_name: user.display_name,
            role: user.role,
        },
    };

    Ok(Json(ApiResponse::success_with_message(
        response_data,
        "Login successful".to_string(),
    )))
}
