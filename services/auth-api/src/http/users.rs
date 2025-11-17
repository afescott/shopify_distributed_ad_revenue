use crate::http::{types::*, ApiContext, AppError, AppResult};
use axum::{
    extract::{Path, Query},
    http::StatusCode,
    routing::get,
    Extension, Json, Router,
};
use sha2::{Digest, Sha256};
use uuid::Uuid;

pub fn users_router() -> Router {
    Router::new()
        .route("/users", get(list_users).post(create_user))
        .route(
            "/users/:id",
            get(get_user).put(update_user).delete(delete_user),
        )
}

// List all users for a merchant
// TODO: Add middleware - Admin only (or Manager can see their merchant's users)
async fn list_users(
    Extension(ctx): Extension<ApiContext>,
    Query(params): Query<ListUsersParams>,
) -> AppResult<UserListResponse> {
    let limit = params.limit.unwrap_or(50).min(100);
    let offset = params.offset.unwrap_or(0);

    // Get total count
    let total: i64 = sqlx::query_scalar::<_, i64>(
        r#"
        SELECT COUNT(*) 
        FROM users 
        WHERE merchant_id = $1
        "#,
    )
    .bind(params.merchant_id)
    .fetch_one(&ctx.db)
    .await?;

    // Get users (excluding password_hash for security)
    let users = sqlx::query_as::<_, UserResponse>(
        r#"
        SELECT 
            id,
            merchant_id,
            email,
            display_name,
            role,
            shopify_user_id,
            last_login_at,
            is_active,
            created_at,
            updated_at
        FROM users
        WHERE merchant_id = $1
        ORDER BY created_at DESC
        LIMIT $2 OFFSET $3
        "#,
    )
    .bind(params.merchant_id)
    .bind(limit as i64)
    .bind(offset as i64)
    .fetch_all(&ctx.db)
    .await?;

    Ok(Json(UserListResponse {
        users,
        total,
        limit,
        offset,
    }))
}

// Get a specific user
// TODO: Add middleware - Admin only (or user can see their own profile)
async fn get_user(
    Extension(ctx): Extension<ApiContext>,
    Path(id): Path<Uuid>,
) -> AppResult<UserResponse> {
    let user = sqlx::query_as::<_, UserResponse>(
        r#"
        SELECT 
            id,
            merchant_id,
            email,
            display_name,
            role,
            shopify_user_id,
            last_login_at,
            is_active,
            created_at,
            updated_at
        FROM users
        WHERE id = $1
        "#,
    )
    .bind(id)
    .fetch_optional(&ctx.db)
    .await?
    .ok_or(AppError::NotFound)?;

    Ok(Json(user))
}

// Create a new user (viewer role only - admin/manager must be created via SQL)
// TODO: Add middleware - ADMIN ONLY
async fn create_user(
    Extension(ctx): Extension<ApiContext>,
    Json(req): Json<CreateUserRequest>,
) -> Result<(StatusCode, Json<UserResponse>), AppError> {
    // Validate email
    crate::misc::validator::validate_email(&req.email)?;

    // Validate password if provided
    if let Some(ref password) = req.password {
        crate::misc::validator::validate_password(password)?;
    }

    // Check if user already exists
    let existing: Option<Uuid> = sqlx::query_scalar(
        "SELECT id FROM users WHERE merchant_id = $1 AND email = $2",
    )
    .bind(req.merchant_id)
    .bind(&req.email)
    .fetch_optional(&ctx.db)
    .await?;

    if existing.is_some() {
        return Err(AppError::Validation(
            "User with this email already exists for this merchant".to_string(),
        ));
    }

    // Hash password if provided
    let password_hash = req.password.map(|password| {
        let mut hasher = Sha256::new();
        hasher.update(password.as_bytes());
        format!("{:x}", hasher.finalize())
    });

    // Always create as viewer - admin/manager roles must be set via SQL scripts
    let role = "viewer".to_string();

    // Insert user
    let user = sqlx::query_as::<_, UserResponse>(
        r#"
        INSERT INTO users (
            merchant_id,
            email,
            password_hash,
            display_name,
            role,
            shopify_user_id,
            is_active
        )
        VALUES ($1, $2, $3, $4, $5, $6, $7)
        RETURNING 
            id,
            merchant_id,
            email,
            display_name,
            role,
            shopify_user_id,
            last_login_at,
            is_active,
            created_at,
            updated_at
        "#,
    )
    .bind(req.merchant_id)
    .bind(&req.email)
    .bind(password_hash)
    .bind(req.display_name)
    .bind(&role)
    .bind(req.shopify_user_id)
    .bind(req.is_active.unwrap_or(true))
    .fetch_one(&ctx.db)
    .await?;

    Ok((StatusCode::CREATED, Json(user)))
}

// Update a user (display_name and password only - no role/is_active changes)
// TODO: Add middleware - Admin only (or user can update their own profile - limited fields)
async fn update_user(
    Extension(ctx): Extension<ApiContext>,
    Path(id): Path<Uuid>,
    Json(req): Json<UpdateUserRequest>,
) -> AppResult<UserResponse> {
    // Validate password if provided
    if let Some(ref password) = req.password {
        crate::misc::validator::validate_password(password)?;
    }

    // Role changes not allowed via API - must use SQL scripts
    // This prevents privilege escalation attacks

    // Build password hash if password is being updated
    let password_hash = req.password.map(|password| {
        let mut hasher = Sha256::new();
        hasher.update(password.as_bytes());
        format!("{:x}", hasher.finalize())
    });

    // Update user (role and is_active changes not allowed via API)
    let user = sqlx::query_as::<_, UserResponse>(
        r#"
        UPDATE users
        SET 
            display_name = COALESCE($2, display_name),
            password_hash = COALESCE($3, password_hash),
            updated_at = NOW()
        WHERE id = $1
        RETURNING 
            id,
            merchant_id,
            email,
            display_name,
            role,
            shopify_user_id,
            last_login_at,
            is_active,
            created_at,
            updated_at
        "#,
    )
    .bind(id)
    .bind(req.display_name)
    .bind(password_hash)
    .fetch_optional(&ctx.db)
    .await?
    .ok_or(AppError::NotFound)?;

    Ok(Json(user))
}

// Delete (soft delete) a user
// TODO: Add middleware - ADMIN ONLY
async fn delete_user(
    Extension(ctx): Extension<ApiContext>,
    Path(id): Path<Uuid>,
) -> Result<StatusCode, AppError> {
    // Soft delete by setting is_active to false
    // This preserves audit trail while preventing login
    let result = sqlx::query(
        r#"
        UPDATE users
        SET is_active = false, updated_at = NOW()
        WHERE id = $1
        "#,
    )
    .bind(id)
    .execute(&ctx.db)
    .await?;

    if result.rows_affected() == 0 {
        return Err(AppError::NotFound);
    }

    Ok(StatusCode::NO_CONTENT)
}

