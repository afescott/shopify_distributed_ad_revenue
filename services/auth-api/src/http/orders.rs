use crate::http::{types::*, ApiContext, AppError, AppResult};
use axum::{
    extract::{Path, Query},
    http::StatusCode,
    routing::get,
    Extension, Json, Router,
};

pub fn orders_router() -> Router {
    Router::new()
        .route("/orders", get(list_orders).post(create_order))
        .route(
            "/orders/:id",
            get(get_order).put(update_order).delete(delete_order),
        )
}

async fn list_orders(
    Extension(ctx): Extension<ApiContext>,
    Query(params): Query<ListOrdersParams>,
) -> AppResult<OrderListResponse> {
    eprintln!(
        "Listing orders: merchant_id={}, limit={:?}, offset={:?}",
        params.merchant_id, params.limit, params.offset
    );

    let limit = params.limit.unwrap_or(50).min(100);
    let offset = params.offset.unwrap_or(0);

    // Get total count
    let total: i64 = sqlx::query_scalar::<_, Option<i64>>(
        r#"
        SELECT COUNT(*) as count
        FROM orders 
        WHERE merchant_id = $1
        "#,
    )
    .bind(params.merchant_id)
    .fetch_one(&ctx.db)
    .await?
    .unwrap_or(0);

    // Get orders
    let orders = sqlx::query_as::<_, Order>(
        r#"
        SELECT 
            id,
            merchant_id,
            shopify_order_id,
            name,
            processed_at,
            currency,
            subtotal_price,
            total_price,
            total_discounts,
            total_shipping_price_set_amount,
            total_tax,
            financial_status,
            cancelled_at,
            created_at,
            updated_at
        FROM orders
        WHERE merchant_id = $1 
            AND ($2::text IS NULL OR financial_status = $2)
        ORDER BY processed_at DESC NULLS LAST, created_at DESC
        LIMIT $3 OFFSET $4
        "#,
    )
    .bind(params.merchant_id)
    .bind(params.financial_status)
    .bind(limit)
    .bind(offset)
    .fetch_all(&ctx.db)
    .await?;

    eprintln!("Found {} orders (total: {})", orders.len(), total);

    Ok(Json(OrderListResponse {
        orders,
        total,
        limit,
        offset,
    }))
}

async fn get_order(
    Extension(ctx): Extension<ApiContext>,
    Path(id): Path<i64>,
) -> AppResult<Order> {
    eprintln!("Getting order: id={}", id);

    let order = sqlx::query_as::<_, Order>(
        r#"
        SELECT 
            id,
            merchant_id,
            shopify_order_id,
            name,
            processed_at,
            currency,
            subtotal_price,
            total_price,
            total_discounts,
            total_shipping_price_set_amount,
            total_tax,
            financial_status,
            cancelled_at,
            created_at,
            updated_at
        FROM orders
        WHERE id = $1
        "#,
    )
    .bind(id)
    .fetch_optional(&ctx.db)
    .await?
    .ok_or(AppError::NotFound)?;

    Ok(Json(order))
}

async fn create_order(
    Extension(ctx): Extension<ApiContext>,
    Json(payload): Json<CreateOrderRequest>,
) -> AppResult<Order> {
    eprintln!(
        "Creating order: merchant_id={}, shopify_order_id={}, name={:?}",
        payload.merchant_id, payload.shopify_order_id, payload.name
    );

    // Check if order already exists
    let existing = sqlx::query_scalar::<_, Option<i64>>(
        r#"
        SELECT id FROM orders 
        WHERE merchant_id = $1 AND shopify_order_id = $2
        "#,
    )
    .bind(payload.merchant_id)
    .bind(payload.shopify_order_id)
    .fetch_optional(&ctx.db)
    .await?;

    eprintln!("Order existence check: {:?}", existing);

    if existing.is_some() {
        return Err(AppError::Validation("Order already exists".to_string()));
    }

    eprintln!("Inserting order into database...");
    let order = sqlx::query_as::<_, Order>(
        r#"
        INSERT INTO orders (
            merchant_id, shopify_order_id, name, processed_at, currency,
            subtotal_price, total_price, total_discounts, 
            total_shipping_price_set_amount, total_tax, financial_status
        )
        VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11)
        RETURNING id, merchant_id, shopify_order_id, name, processed_at, currency,
                  subtotal_price, total_price, total_discounts, 
                  total_shipping_price_set_amount, total_tax, financial_status,
                  cancelled_at, created_at, updated_at
        "#,
    )
    .bind(payload.merchant_id)
    .bind(payload.shopify_order_id)
    .bind(payload.name)
    .bind(payload.processed_at)
    .bind(payload.currency)
    .bind(payload.subtotal_price)
    .bind(payload.total_price)
    .bind(payload.total_discounts)
    .bind(payload.total_shipping_price_set_amount)
    .bind(payload.total_tax)
    .bind(payload.financial_status)
    .fetch_one(&ctx.db)
    .await?;

    eprintln!("Order created successfully: id={}", order.id);
    Ok(Json(order))
}

async fn update_order(
    Extension(ctx): Extension<ApiContext>,
    Path(id): Path<i64>,
    Json(payload): Json<UpdateOrderRequest>,
) -> AppResult<Order> {
    eprintln!(
        "Updating order: id={}, name={:?}, financial_status={:?}",
        id, payload.name, payload.financial_status
    );

    let order = sqlx::query_as::<_, Order>(
        r#"
        UPDATE orders 
        SET 
            name = COALESCE($2, name),
            financial_status = COALESCE($3, financial_status),
            cancelled_at = COALESCE($4, cancelled_at),
            updated_at = NOW()
        WHERE id = $1
        RETURNING id, merchant_id, shopify_order_id, name, processed_at, currency,
                  subtotal_price, total_price, total_discounts, 
                  total_shipping_price_set_amount, total_tax, financial_status,
                  cancelled_at, created_at, updated_at
        "#,
    )
    .bind(id)
    .bind(payload.name)
    .bind(payload.financial_status)
    .bind(payload.cancelled_at)
    .fetch_optional(&ctx.db)
    .await?
    .ok_or(AppError::NotFound)?;

    eprintln!("Order updated successfully: id={}", order.id);
    Ok(Json(order))
}

async fn delete_order(
    Extension(ctx): Extension<ApiContext>,
    Path(id): Path<i64>,
) -> Result<StatusCode, AppError> {
    eprintln!("Deleting order: id={}", id);

    let result = sqlx::query(
        r#"
        DELETE FROM orders 
        WHERE id = $1
        "#,
    )
    .bind(id)
    .execute(&ctx.db)
    .await?;

    if result.rows_affected() == 0 {
        return Err(AppError::NotFound);
    }

    eprintln!("Order deleted successfully: id={}", id);
    Ok(StatusCode::NO_CONTENT)
}
