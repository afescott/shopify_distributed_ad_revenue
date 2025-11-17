use crate::http::{types::*, ApiContext, AppError, AppResult};
use axum::{
    extract::{Path, Query},
    http::StatusCode,
    routing::get,
    Extension, Json, Router,
};

pub fn inventory_router() -> Router {
    Router::new()
        .route("/inventory", get(list_items).post(create_item))
        .route(
            "/inventory/:id",
            get(get_item).put(update_item).delete(delete_item),
        )
}

async fn list_items(
    Extension(ctx): Extension<ApiContext>,
    Query(params): Query<ListInventoryItemsParams>,
) -> AppResult<InventoryItemListResponse> {
    eprintln!(
        "Listing inventory items: merchant_id={}, limit={:?}, offset={:?}",
        params.merchant_id, params.limit, params.offset
    );

    let limit = params.limit.unwrap_or(50).min(100);
    let offset = params.offset.unwrap_or(0);

    // Get total count
    let total: i64 = sqlx::query_scalar::<_, Option<i64>>(
        r#"
        SELECT COUNT(*) as count
        FROM inventory_items 
        WHERE merchant_id = $1
        "#,
    )
    .bind(params.merchant_id)
    .fetch_one(&ctx.db)
    .await?
    .unwrap_or(0);

    // Get inventory items
    let items = sqlx::query_as::<_, InventoryItem>(
        r#"
        SELECT 
            id,
            merchant_id,
            shopify_inventory_item_id,
            shopify_variant_id,
            created_at,
            updated_at
        FROM inventory_items
        WHERE merchant_id = $1
        ORDER BY updated_at DESC
        LIMIT $2 OFFSET $3
        "#,
    )
    .bind(params.merchant_id)
    .bind(limit)
    .bind(offset)
    .fetch_all(&ctx.db)
    .await?;

    eprintln!("Found {} inventory items (total: {})", items.len(), total);

    Ok(Json(InventoryItemListResponse {
        items,
        total,
        limit,
        offset,
    }))
}

async fn get_item(
    Extension(ctx): Extension<ApiContext>,
    Path(id): Path<uuid::Uuid>,
) -> AppResult<InventoryItem> {
    eprintln!("Getting inventory item: id={}", id);

    let item = sqlx::query_as::<_, InventoryItem>(
        r#"
        SELECT 
            id,
            merchant_id,
            shopify_inventory_item_id,
            shopify_variant_id,
            created_at,
            updated_at
        FROM inventory_items
        WHERE id = $1
        "#,
    )
    .bind(id)
    .fetch_optional(&ctx.db)
    .await?
    .ok_or(AppError::NotFound)?;

    Ok(Json(item))
}

async fn create_item(
    Extension(ctx): Extension<ApiContext>,
    Json(payload): Json<CreateInventoryItemRequest>,
) -> AppResult<InventoryItem> {
    eprintln!(
        "Creating inventory item: merchant_id={}, shopify_inventory_item_id={}, shopify_variant_id={:?}",
        payload.merchant_id, payload.shopify_inventory_item_id, payload.shopify_variant_id
    );

    // Check if item already exists
    let existing = sqlx::query_scalar::<_, Option<uuid::Uuid>>(
        r#"
        SELECT id FROM inventory_items 
        WHERE merchant_id = $1 AND shopify_inventory_item_id = $2
        "#,
    )
    .bind(payload.merchant_id)
    .bind(payload.shopify_inventory_item_id)
    .fetch_optional(&ctx.db)
    .await?;

    eprintln!("Inventory item existence check: {:?}", existing);

    if existing.is_some() {
        return Err(AppError::Validation(
            "Inventory item already exists".to_string(),
        ));
    }

    eprintln!("Inserting inventory item into database...");
    let item = sqlx::query_as::<_, InventoryItem>(
        r#"
        INSERT INTO inventory_items (
            merchant_id, shopify_inventory_item_id, shopify_variant_id
        )
        VALUES ($1, $2, $3)
        RETURNING id, merchant_id, shopify_inventory_item_id, shopify_variant_id,
                  created_at, updated_at
        "#,
    )
    .bind(payload.merchant_id)
    .bind(payload.shopify_inventory_item_id)
    .bind(payload.shopify_variant_id)
    .fetch_one(&ctx.db)
    .await?;

    eprintln!("Inventory item created successfully: id={}", item.id);
    Ok(Json(item))
}

async fn update_item(
    Extension(ctx): Extension<ApiContext>,
    Path(id): Path<uuid::Uuid>,
    Json(payload): Json<UpdateInventoryItemRequest>,
) -> AppResult<InventoryItem> {
    eprintln!(
        "Updating inventory item: id={}, shopify_variant_id={:?}",
        id, payload.shopify_variant_id
    );

    let item = sqlx::query_as::<_, InventoryItem>(
        r#"
        UPDATE inventory_items 
        SET 
            shopify_variant_id = COALESCE($2, shopify_variant_id),
            updated_at = NOW()
        WHERE id = $1
        RETURNING id, merchant_id, shopify_inventory_item_id, shopify_variant_id,
                  created_at, updated_at
        "#,
    )
    .bind(id)
    .bind(payload.shopify_variant_id)
    .fetch_optional(&ctx.db)
    .await?
    .ok_or(AppError::NotFound)?;

    eprintln!("Inventory item updated successfully: id={}", item.id);
    Ok(Json(item))
}

async fn delete_item(
    Extension(ctx): Extension<ApiContext>,
    Path(id): Path<uuid::Uuid>,
) -> Result<StatusCode, AppError> {
    eprintln!("Deleting inventory item: id={}", id);

    let result = sqlx::query(
        r#"
        DELETE FROM inventory_items 
        WHERE id = $1
        "#,
    )
    .bind(id)
    .execute(&ctx.db)
    .await?;

    if result.rows_affected() == 0 {
        return Err(AppError::NotFound);
    }

    eprintln!("Inventory item deleted successfully: id={}", id);
    Ok(StatusCode::NO_CONTENT)
}
