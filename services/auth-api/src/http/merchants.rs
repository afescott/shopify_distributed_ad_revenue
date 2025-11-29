use crate::http::{types::*, ApiContext, AppError, AppResult};
use axum::{
    extract::Path,
    http::StatusCode,
    routing::{delete, get, post},
    Extension, Json, Router,
};
use uuid::Uuid;

/// Create a merchant (can be used by HTTP handlers and tests)
pub async fn create_merchant(
    db: &sqlx::PgPool,
    payload: CreateMerchantRequest,
) -> Result<Merchant, AppError> {
    // Check if merchant already exists by shop_domain
    let existing = sqlx::query_scalar::<_, Option<Uuid>>(
        r#"
        SELECT id FROM merchants 
        WHERE shop_domain = $1
        "#,
    )
    .bind(&payload.shop_domain)
    .fetch_optional(db)
    .await?;

    if existing.is_some() {
        return Err(AppError::Validation("Merchant with this shop_domain already exists".to_string()));
    }

    // Insert merchant
    let merchant = sqlx::query_as::<_, Merchant>(
        r#"
        INSERT INTO merchants (
            id, shop_domain, shop_name, shop_currency, timezone, created_at, updated_at
        )
        VALUES ($1, $2, $3, $4, $5, NOW(), NOW())
        RETURNING id, shop_domain, shop_name, shop_currency, timezone, created_at, updated_at, deleted_at
        "#,
    )
    .bind(payload.id.unwrap_or_else(|| Uuid::new_v4()))
    .bind(&payload.shop_domain)
    .bind(payload.shop_name.as_ref())
    .bind(payload.shop_currency.as_ref())
    .bind(payload.timezone.as_ref())
    .fetch_one(db)
    .await?;

    Ok(merchant)
}

/// Get a merchant by ID (can be used by HTTP handlers and tests)
pub async fn get_merchant(db: &sqlx::PgPool, id: Uuid) -> Result<Merchant, AppError> {
    let merchant = sqlx::query_as::<_, Merchant>(
        r#"
        SELECT id, shop_domain, shop_name, shop_currency, timezone, created_at, updated_at, deleted_at
        FROM merchants
        WHERE id = $1 AND deleted_at IS NULL
        "#,
    )
    .bind(id)
    .fetch_optional(db)
    .await?
    .ok_or(AppError::NotFound)?;

    Ok(merchant)
}

/// Delete a merchant (soft delete by setting deleted_at)
pub async fn delete_merchant(db: &sqlx::PgPool, id: Uuid) -> Result<(), AppError> {
    let result = sqlx::query(
        r#"
        UPDATE merchants 
        SET deleted_at = NOW(), updated_at = NOW()
        WHERE id = $1 AND deleted_at IS NULL
        "#,
    )
    .bind(id)
    .execute(db)
    .await?;

    if result.rows_affected() == 0 {
        return Err(AppError::NotFound);
    }

    Ok(())
}

pub fn merchants_router() -> Router {
    Router::new()
        .route("/merchants", post(create_merchant_handler))
        .route("/merchants/:id", get(get_merchant_handler).delete(delete_merchant_handler))
}

async fn create_merchant_handler(
    Extension(ctx): Extension<ApiContext>,
    Json(payload): Json<CreateMerchantRequest>,
) -> AppResult<Merchant> {
    eprintln!(
        "Creating merchant: shop_domain={}, shop_name={:?}",
        payload.shop_domain, payload.shop_name
    );

    let merchant = create_merchant(&ctx.db, payload).await?;

    eprintln!("Merchant created successfully: id={}", merchant.id);
    Ok(Json(merchant))
}

async fn get_merchant_handler(
    Extension(ctx): Extension<ApiContext>,
    Path(id): Path<Uuid>,
) -> AppResult<Merchant> {
    eprintln!("Getting merchant: id={}", id);

    let merchant = get_merchant(&ctx.db, id).await?;
    Ok(Json(merchant))
}

async fn delete_merchant_handler(
    Extension(ctx): Extension<ApiContext>,
    Path(id): Path<Uuid>,
) -> Result<StatusCode, AppError> {
    eprintln!("Deleting merchant: id={}", id);

    delete_merchant(&ctx.db, id).await?;

    eprintln!("Merchant deleted successfully: id={}", id);
    Ok(StatusCode::NO_CONTENT)
}

