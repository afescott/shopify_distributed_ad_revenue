use anyhow::Context;
use axum::{
    extract::State,
    http::StatusCode,
    response::Json,
    routing::{get, post},
    Router,
};
use clap::Parser;
use reqwest::Client;
use std::sync::Arc;
use uuid::Uuid;
use rust_decimal::Decimal;
use chrono::{DateTime, Utc};
use rdkafka::producer::FutureProducer;
use serde::{Deserialize, Serialize};
use tokio_cron_scheduler::{Job, JobScheduler};

mod shopify;

use shopify::{ShopifyClient, ShopifyProduct, ShopifyOrder};
use lib_shopify::kafka::create_producer;

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// Shopify store name
    #[arg(short, long, env = "SHOPIFY_STORE_NAME")]
    store_name: String,

    /// Shopify access token
    #[arg(short, long, env = "SHOPIFY_ACCESS_TOKEN")]
    access_token: String,

    /// Shopify API version
    #[arg(short, long, default_value = "2025-10", env = "SHOPIFY_API_VERSION")]
    api_version: String,

    /// Auth API base URL
    #[arg(long, default_value = "http://localhost:8080", env = "AUTH_API_URL")]
    auth_api_url: String,

    /// Merchant ID (UUID)
    #[arg(short, long, env = "MERCHANT_ID")]
    merchant_id: Uuid,

    /// Kafka brokers (comma-separated)
    #[arg(long, env = "KAFKA_BROKERS")]
    kafka_brokers: Option<String>,

    /// HTTP server port
    #[arg(long, default_value = "8081", env = "HTTP_PORT")]
    http_port: u16,

    /// Enable periodic product sync (cron expression, e.g., "0 */6 * * *" for every 6 hours)
    #[arg(long, env = "PRODUCTS_SYNC_CRON")]
    products_sync_cron: Option<String>,

    /// Enable periodic order sync (cron expression, e.g., "0 */1 * * *" for every hour)
    #[arg(long, env = "ORDERS_SYNC_CRON")]
    orders_sync_cron: Option<String>,
}

#[derive(Clone)]
struct AppContext {
    shopify_client: Arc<ShopifyClient>,
    http_client: Arc<Client>,
    auth_api_url: String,
    merchant_id: Uuid,
    kafka_producer: Option<Arc<FutureProducer>>,
}

#[derive(Serialize)]
struct SyncResponse {
    success: bool,
    message: String,
}

#[derive(Deserialize)]
struct SyncRequest {
    limit: Option<u32>,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let args = Args::parse();

    let shopify_client = Arc::new(ShopifyClient::new(
        args.store_name,
        args.access_token,
        args.api_version,
    ));

    let http_client = Arc::new(Client::new());
    let auth_api_url = args.auth_api_url.trim_end_matches('/').to_string();

    // Initialize Kafka producer if brokers are provided
    let kafka_producer = if let Some(brokers) = &args.kafka_brokers {
        Some(create_producer(brokers).context("Failed to create Kafka producer")?)
    } else {
        None
    };

    let app_context = AppContext {
        shopify_client: shopify_client.clone(),
        http_client: http_client.clone(),
        auth_api_url: auth_api_url.clone(),
        merchant_id: args.merchant_id,
        kafka_producer,
    };

    // Setup scheduler for periodic syncs
    let scheduler = JobScheduler::new().await?;
    
    if let Some(cron) = &args.products_sync_cron {
        let ctx = app_context.clone();
        scheduler
            .add(
                Job::new_async(cron.as_str(), move |_uuid, _l| {
                    let ctx = ctx.clone();
                    Box::pin(async move {
                        println!("🔄 Scheduled product sync triggered");
                        if let Err(e) = sync_products_internal(&ctx, None).await {
                            eprintln!("❌ Scheduled product sync failed: {}", e);
                        }
                    })
                })?
            )
            .await?;
        println!("✅ Product sync scheduled: {}", cron);
    }

    if let Some(cron) = &args.orders_sync_cron {
        let ctx = app_context.clone();
        scheduler
            .add(
                Job::new_async(cron.as_str(), move |_uuid, _l| {
                    let ctx = ctx.clone();
                    Box::pin(async move {
                        println!("🔄 Scheduled order sync triggered");
                        if let Err(e) = sync_orders_internal(&ctx, None).await {
                            eprintln!("❌ Scheduled order sync failed: {}", e);
                        }
                    })
                })?
            )
            .await?;
        println!("✅ Order sync scheduled: {}", cron);
    }

    scheduler.start().await?;

    // Setup HTTP API
    let app = Router::new()
        .route("/health", get(health_check))
        .route("/api/v1/sync/products", post(trigger_sync_products))
        .route("/api/v1/sync/orders", post(trigger_sync_orders))
        .with_state(app_context);

    let listener = tokio::net::TcpListener::bind(format!("0.0.0.0:{}", args.http_port))
        .await
        .context("Failed to bind HTTP server")?;

    println!("🚀 Shopify Consumer service running on port {}", args.http_port);
    println!("📡 Health check: http://localhost:{}/health", args.http_port);
    println!("🔄 Sync endpoints:");
    println!("   POST http://localhost:{}/api/v1/sync/products", args.http_port);
    println!("   POST http://localhost:{}/api/v1/sync/orders", args.http_port);

    axum::serve(listener, app).await?;

    Ok(())
}

async fn health_check() -> Json<SyncResponse> {
    Json(SyncResponse {
        success: true,
        message: "Service is healthy".to_string(),
    })
}

async fn trigger_sync_products(
    State(ctx): State<AppContext>,
    Json(req): Json<SyncRequest>,
) -> Result<Json<SyncResponse>, (StatusCode, Json<SyncResponse>)> {
    match sync_products_internal(&ctx, req.limit).await {
        Ok(_) => Ok(Json(SyncResponse {
            success: true,
            message: "Product sync completed".to_string(),
        })),
        Err(e) => Err((
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(SyncResponse {
                success: false,
                message: format!("Product sync failed: {}", e),
            }),
        )),
    }
}

async fn trigger_sync_orders(
    State(ctx): State<AppContext>,
    Json(req): Json<SyncRequest>,
) -> Result<Json<SyncResponse>, (StatusCode, Json<SyncResponse>)> {
    match sync_orders_internal(&ctx, req.limit).await {
        Ok(_) => Ok(Json(SyncResponse {
            success: true,
            message: "Order sync completed".to_string(),
        })),
        Err(e) => Err((
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(SyncResponse {
                success: false,
                message: format!("Order sync failed: {}", e),
            }),
        )),
    }
}

async fn sync_products_internal(ctx: &AppContext, limit: Option<u32>) -> anyhow::Result<()> {
    sync_products(&ctx.http_client, &ctx.shopify_client, &ctx.auth_api_url, ctx.merchant_id, limit, ctx).await
}

async fn sync_orders_internal(ctx: &AppContext, limit: Option<u32>) -> anyhow::Result<()> {
    sync_orders(&ctx.http_client, &ctx.shopify_client, &ctx.auth_api_url, ctx.merchant_id, limit, ctx).await
}

async fn sync_products(
    http_client: &Client,
    shopify_client: &ShopifyClient,
    auth_api_url: &str,
    merchant_id: Uuid,
    limit: Option<u32>,
    app_context: &AppContext,
) -> anyhow::Result<()> {
    let products = shopify_client
        .get_products(limit, None)
        .await
        .context("Failed to fetch products from Shopify")?;

    println!("Fetched {} products from Shopify", products.len());

    for product in products {
        // Create product via auth API
        let product_payload = serde_json::json!({
            "merchant_id": merchant_id,
            "shopify_product_id": product.id,
            "title": product.title,
            "product_type": product.product_type,
            "status": product.status,
        });

        let url = format!("{}/api/v1/products", auth_api_url);
        let response = http_client
            .post(&url)
            .json(&product_payload)
            .send()
            .await?;

        if response.status().is_success() {
            println!("✓ Synced product: {} (Shopify ID: {})", product.title, product.id);
        } else if response.status() == 400 {
            let error_text = response.text().await?;
            if error_text.contains("already exists") {
                println!("⊘ Product {} already exists, skipping", product.id);
            } else {
                eprintln!("✗ Error syncing product {}: {}", product.id, error_text);
            }
        } else {
            let error_text = response.text().await?;
            eprintln!("✗ Error syncing product {}: {}", product.id, error_text);
        }

        // Sync variants
        for variant in product.variants {
            // Note: Variants would need their own endpoint or be handled differently
            // For now, we'll just log them
            println!("  - Variant: {} (SKU: {:?})", variant.title, variant.sku);
        }
    }

    Ok(())
}

async fn sync_orders(
    http_client: &Client,
    shopify_client: &ShopifyClient,
    auth_api_url: &str,
    merchant_id: Uuid,
    limit: Option<u32>,
    app_context: &AppContext,
) -> anyhow::Result<()> {
    let orders = shopify_client
        .get_orders(limit, None, Some("any"), None)
        .await
        .context("Failed to fetch orders from Shopify")?;

    println!("Fetched {} orders from Shopify", orders.len());

    for order in orders {
        // Parse processed_at
        let processed_at = order.processed_at
            .as_ref()
            .and_then(|s| DateTime::parse_from_rfc3339(s).ok())
            .map(|dt| dt.with_timezone(&Utc));

        // Parse decimal values
        let subtotal_price = order.subtotal_price.parse::<Decimal>().ok();
        let total_price = order.total_price.parse::<Decimal>().ok();
        let total_discounts = order.total_discounts.parse::<Decimal>().ok();
        let total_shipping_price_set_amount = order.total_shipping_price_set
            .shop_money
            .amount
            .parse::<Decimal>()
            .ok();
        let total_tax = order.total_tax.parse::<Decimal>().ok();

        let order_payload = serde_json::json!({
            "merchant_id": merchant_id,
            "shopify_order_id": order.id,
            "name": order.name,
            "processed_at": processed_at.map(|dt| dt.to_rfc3339()),
            "currency": order.currency,
            "subtotal_price": subtotal_price.map(|d| d.to_string()),
            "total_price": total_price.map(|d| d.to_string()),
            "total_discounts": total_discounts.map(|d| d.to_string()),
            "total_shipping_price_set_amount": total_shipping_price_set_amount.map(|d| d.to_string()),
            "total_tax": total_tax.map(|d| d.to_string()),
            "financial_status": order.financial_status,
        });

        let url = format!("{}/api/v1/orders", auth_api_url);
        let response = http_client
            .post(&url)
            .json(&order_payload)
            .send()
            .await?;

        if response.status().is_success() {
            println!("✓ Synced order: {} (Shopify ID: {})", order.name, order.id);
        } else if response.status() == 400 {
            let error_text = response.text().await?;
            if error_text.contains("already exists") {
                println!("⊘ Order {} already exists, skipping", order.id);
            } else {
                eprintln!("✗ Error syncing order {}: {}", order.id, error_text);
            }
        } else {
            let error_text = response.text().await?;
            eprintln!("✗ Error syncing order {}: {}", order.id, error_text);
        }
    }

    Ok(())
}

