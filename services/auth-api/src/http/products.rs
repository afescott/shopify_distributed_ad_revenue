use crate::http::{types::*, ApiContext, AppError, AppResult};
use axum::{
    extract::{Path, Query},
    http::StatusCode,
    routing::get,
    Extension, Json, Router,
};

pub fn products_router() -> Router {
    Router::new()
        .route("/products", get(list_products).post(create_product))
        .route(
            "/products/:id",
            get(get_product).put(update_product).delete(delete_product),
        )
}

async fn list_products(
    Extension(ctx): Extension<ApiContext>,
    Query(params): Query<ListProductsParams>,
) -> AppResult<ProductListResponse> {
    eprintln!("Listing products: merchant_id={}, limit={:?}, offset={:?}", 
              params.merchant_id, params.limit, params.offset);
    
    let limit = params.limit.unwrap_or(50).min(100);
    let offset = params.offset.unwrap_or(0);

    // Get total count
    let total: i64 = sqlx::query_scalar::<_, Option<i64>>(
        r#"
        SELECT COUNT(*) as count
        FROM products 
        WHERE merchant_id = $1 AND deleted_at IS NULL
        "#,
    )
    .bind(params.merchant_id)
    .fetch_one(&ctx.db)
    .await?
    .unwrap_or(0);

    // Get products
    let products = sqlx::query_as::<_, Product>(
        r#"
        SELECT 
            id,
            merchant_id,
            shopify_product_id,
            title,
            product_type,
            status,
            created_at,
            updated_at,
            deleted_at
        FROM products
        WHERE merchant_id = $1 
            AND deleted_at IS NULL
            AND ($2::text IS NULL OR product_type = $2)
            AND ($3::text IS NULL OR status = $3)
        ORDER BY updated_at DESC
        LIMIT $4 OFFSET $5
        "#,
    )
    .bind(params.merchant_id)
    .bind(params.product_type)
    .bind(params.status)
    .bind(limit)
    .bind(offset)
    .fetch_all(&ctx.db)
    .await?;

    // Get variants for each product
    let mut products_with_variants = Vec::new();
    for product in products {
        let variants = sqlx::query_as::<_, Variant>(
            r#"
            SELECT 
                id,
                merchant_id,
                shopify_variant_id,
                shopify_product_id,
                sku,
                title,
                barcode,
                weight,
                weight_unit,
                created_at,
                updated_at
            FROM variants
            WHERE merchant_id = $1 AND shopify_product_id = $2
            ORDER BY created_at
            "#,
        )
        .bind(product.merchant_id)
        .bind(product.shopify_product_id)
        .fetch_all(&ctx.db)
        .await?;

        let product_with_variants = ProductWithVariants {
            product: product.clone(),
            variants,
            variant_count: 0, // Will be set correctly when we have the actual count
        };

        products_with_variants.push(product_with_variants);
    }

    Ok(Json(ProductListResponse {
        products: products_with_variants,
        total,
        limit,
        offset,
    }))
}

async fn get_product(
    Extension(ctx): Extension<ApiContext>,
    Path(id): Path<uuid::Uuid>,
) -> AppResult<ProductWithVariants> {
    eprintln!("Getting product: id={}", id);
    
    // Get product
    let product = sqlx::query_as::<_, Product>(
        r#"
        SELECT 
            id,
            merchant_id,
            shopify_product_id,
            title,
            product_type,
            status,
            created_at,
            updated_at,
            deleted_at
        FROM products
        WHERE id = $1 AND deleted_at IS NULL
        "#,
    )
    .bind(id)
    .fetch_optional(&ctx.db)
    .await?
    .ok_or(AppError::NotFound)?;

    // Get variants
    let variants = sqlx::query_as::<_, Variant>(
        r#"
        SELECT 
            id,
            merchant_id,
            shopify_variant_id,
            shopify_product_id,
            sku,
            title,
            barcode,
            weight,
            weight_unit,
            created_at,
            updated_at
        FROM variants
        WHERE merchant_id = $1 AND shopify_product_id = $2
        ORDER BY created_at
        "#,
    )
    .bind(product.merchant_id)
    .bind(product.shopify_product_id)
    .fetch_all(&ctx.db)
    .await?;

    let product_with_variants = ProductWithVariants {
        product: product.clone(),
        variants: variants.clone(),
        variant_count: variants.len() as i64,
    };

    Ok(Json(product_with_variants))
}

async fn create_product(
    Extension(ctx): Extension<ApiContext>,
    Json(payload): Json<CreateProductRequest>,
) -> AppResult<Product> {
    eprintln!("Creating product: merchant_id={}, shopify_product_id={}, title={:?}", 
              payload.merchant_id, payload.shopify_product_id, payload.title);
    
    // Check if product already exists
    let existing = sqlx::query_scalar::<_, Option<uuid::Uuid>>(
        r#"
        SELECT id FROM products 
        WHERE merchant_id = $1 AND shopify_product_id = $2 AND deleted_at IS NULL
        "#,
    )
    .bind(payload.merchant_id)
    .bind(payload.shopify_product_id)
    .fetch_optional(&ctx.db)
    .await?;
    
    eprintln!("Product existence check: {:?}", existing);

    if existing.is_some() {
        return Err(AppError::Validation("Product already exists".to_string()));
    }

    eprintln!("Inserting product into database...");
    let product = sqlx::query_as::<_, Product>(
        r#"
        INSERT INTO products (merchant_id, shopify_product_id, title, product_type, status)
        VALUES ($1, $2, $3, $4, $5)
        RETURNING id, merchant_id, shopify_product_id, title, product_type, status, created_at, updated_at, deleted_at
        "#,
    )
    .bind(payload.merchant_id)
    .bind(payload.shopify_product_id)
    .bind(payload.title)
    .bind(payload.product_type)
    .bind(payload.status)
    .fetch_one(&ctx.db)
    .await?;

    eprintln!("Product created successfully: id={}", product.id);
    Ok(Json(product))
}

async fn update_product(
    Extension(ctx): Extension<ApiContext>,
    Path(id): Path<uuid::Uuid>,
    Json(payload): Json<UpdateProductRequest>,
) -> AppResult<Product> {
    eprintln!("Updating product: id={}, title={:?}, product_type={:?}, status={:?}", 
              id, payload.title, payload.product_type, payload.status);
    
    let product = sqlx::query_as::<_, Product>(
        r#"
        UPDATE products 
        SET 
            title = COALESCE($2, title),
            product_type = COALESCE($3, product_type),
            status = COALESCE($4, status),
            updated_at = NOW()
        WHERE id = $1 AND deleted_at IS NULL
        RETURNING id, merchant_id, shopify_product_id, title, product_type, status, created_at, updated_at, deleted_at
        "#,
    )
    .bind(id)
    .bind(payload.title)
    .bind(payload.product_type)
    .bind(payload.status)
    .fetch_optional(&ctx.db)
    .await?
    .ok_or(AppError::NotFound)?;

    Ok(Json(product))
}

async fn delete_product(
    Extension(ctx): Extension<ApiContext>,
    Path(id): Path<uuid::Uuid>,
) -> Result<StatusCode, AppError> {
    eprintln!("Deleting product: id={}", id);
    
    let result = sqlx::query(
        r#"
        UPDATE products 
        SET deleted_at = NOW(), updated_at = NOW()
        WHERE id = $1 AND deleted_at IS NULL
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