use super::{ApiContext, AppResult};
use axum::{extract::Query, routing::post, Extension, Json, Router};
use chrono::{DateTime, Utc};
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

mod ad_campaign;
mod courier;
mod shopify_client;

pub fn cost_router() -> Router {
    Router::new().route("/calculate", post(post_calculate))
}

#[derive(Serialize, Deserialize)]
pub struct CalculateProfitParams {
    pub merchant_id: Uuid,
    pub start_date: Option<DateTime<Utc>>,
    pub end_date: Option<DateTime<Utc>>,
}

#[derive(Serialize)]
pub struct ProfitCalculation {
    pub shopify_revenue: Decimal,
    pub shopify_product_cost: Decimal,
    pub ad_cost: Decimal,
    pub courier_cost: Decimal,
    pub manual_cost: Decimal,
    pub profit: Decimal,
}

pub async fn post_calculate(
    Extension(ctx): Extension<ApiContext>,
    Query(params): Query<CalculateProfitParams>,
) -> AppResult<ProfitCalculation> {
    // 1. Get Shopify revenue using helper function
    let shopify_revenue = get_total_revenue(
        &ctx.db,
        params.merchant_id,
        params.start_date,
        params.end_date,
    )
    .await?;

    // 2. Get Shopify product cost (placeholder - would need product cost data)
    // TODO: Replace with actual product cost query when cost data is available
    let shopify_product_cost: Decimal = Decimal::ZERO;

    // 3. Get ad cost (placeholder query - would need ad_cost table)
    // TODO: Replace with actual ad_cost query when ad_cost table is available
    let ad_cost: Decimal = Decimal::ZERO;

    // 4. Get courier cost using helper function
    let courier_cost = get_total_courier_cost(
        &ctx.db,
        params.merchant_id,
        params.start_date,
        params.end_date,
    )
    .await?;

    // 5. Get manual cost (placeholder query - would need manual_cost table)
    // TODO: Replace with actual manual_cost query when manual_cost table is available
    let manual_cost: Decimal = Decimal::ZERO;

    // Calculate profit
    let profit = shopify_revenue - shopify_product_cost - ad_cost - courier_cost - manual_cost;

    Ok(Json(ProfitCalculation {
        shopify_revenue,
        shopify_product_cost,
        ad_cost,
        courier_cost,
        manual_cost,
        profit,
    }))
}

// Helper functions for order calculations (can be used by other modules)
async fn get_total_revenue(
    db: &sqlx::PgPool,
    merchant_id: Uuid,
    start_date: Option<DateTime<Utc>>,
    end_date: Option<DateTime<Utc>>,
) -> Result<Decimal, sqlx::Error> {
    match (start_date, end_date) {
        (Some(start), Some(end)) => {
            sqlx::query_scalar(
                "SELECT COALESCE(SUM(total_price), 0) FROM orders 
                 WHERE merchant_id = $1 AND processed_at >= $2 AND processed_at <= $3",
            )
            .bind(merchant_id)
            .bind(start)
            .bind(end)
            .fetch_one(db)
            .await
        }
        (Some(start), None) => {
            sqlx::query_scalar(
                "SELECT COALESCE(SUM(total_price), 0) FROM orders 
                 WHERE merchant_id = $1 AND processed_at >= $2",
            )
            .bind(merchant_id)
            .bind(start)
            .fetch_one(db)
            .await
        }
        (None, Some(end)) => {
            sqlx::query_scalar(
                "SELECT COALESCE(SUM(total_price), 0) FROM orders 
                 WHERE merchant_id = $1 AND processed_at <= $2",
            )
            .bind(merchant_id)
            .bind(end)
            .fetch_one(db)
            .await
        }
        (None, None) => {
            sqlx::query_scalar(
                "SELECT COALESCE(SUM(total_price), 0) FROM orders WHERE merchant_id = $1",
            )
            .bind(merchant_id)
            .fetch_one(db)
            .await
        }
    }
}

async fn get_total_courier_cost(
    db: &sqlx::PgPool,
    merchant_id: Uuid,
    start_date: Option<DateTime<Utc>>,
    end_date: Option<DateTime<Utc>>,
) -> Result<Decimal, sqlx::Error> {
    match (start_date, end_date) {
        (Some(start), Some(end)) => {
            sqlx::query_scalar(
                "SELECT COALESCE(SUM(total_shipping_price_set_amount), 0) FROM orders 
                 WHERE merchant_id = $1 AND processed_at >= $2 AND processed_at <= $3",
            )
            .bind(merchant_id)
            .bind(start)
            .bind(end)
            .fetch_one(db)
            .await
        }
        (Some(start), None) => {
            sqlx::query_scalar(
                "SELECT COALESCE(SUM(total_shipping_price_set_amount), 0) FROM orders 
                 WHERE merchant_id = $1 AND processed_at >= $2",
            )
            .bind(merchant_id)
            .bind(start)
            .fetch_one(db)
            .await
        }
        (None, Some(end)) => {
            sqlx::query_scalar(
                "SELECT COALESCE(SUM(total_shipping_price_set_amount), 0) FROM orders 
                 WHERE merchant_id = $1 AND processed_at <= $2",
            )
            .bind(merchant_id)
            .bind(end)
            .fetch_one(db)
            .await
        }
        (None, None) => {
            sqlx::query_scalar(
                "SELECT COALESCE(SUM(total_shipping_price_set_amount), 0) FROM orders 
                 WHERE merchant_id = $1",
            )
            .bind(merchant_id)
            .fetch_one(db)
            .await
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use anyhow::Context;
    use std::str::FromStr;
    use sqlx::postgres::PgPoolOptions;

    /// Setup test database connection and run migrations
    ///
    /// Uses TEST_DATABASE_URL environment variable if set, otherwise falls back to:
    /// - DATABASE_URL if set
    /// - Default test database URL (postgresql://postgres:postgres@localhost:5433/shopify_db)
    ///
    /// To start the database for testing, run:
    ///   docker-compose up -d postgres
    async fn setup_test_db() -> anyhow::Result<sqlx::PgPool> {
        // Use test database URL from environment or default
        let database_url = std::env::var("TEST_DATABASE_URL")
            .or_else(|_| std::env::var("DATABASE_URL"))
            .unwrap_or_else(|_| {
                // Default: matches docker-compose.yml configuration
                "postgresql://postgres:postgres@localhost:5433/shopify_db".to_string()
            });

        let db = PgPoolOptions::new()
            .max_connections(5) // Lower for tests
            .connect(&database_url)
            .await
            .with_context(|| {
                format!(
                    "could not connect to test database at: {}\n\
                     Hint: Start the database with: docker-compose up -d postgres",
                    database_url
                )
            })?;

        // Run migrations (path is relative to crate root where Cargo.toml is)
        sqlx::migrate!("./sql/migrations")
            .run(&db)
            .await
            .context("could not run migrations")?;

        Ok(db)
    }

    #[tokio::test]
    async fn test_get_total_revenue() {
        use crate::http::orders::create_order;
        use crate::http::types::CreateOrderRequest;

        let db = setup_test_db().await.expect(
            "Failed to setup test database. Make sure PostgreSQL is running and accessible.",
        );

        use crate::http::merchants::create_merchant;
        use crate::http::types::CreateMerchantRequest;

        let merchant_id = Uuid::new_v4();

        // Create a merchant first (required by foreign key constraint)
        create_merchant(
            &db,
            CreateMerchantRequest {
                id: Some(merchant_id),
                shop_domain: format!("test-merchant-{}.myshopify.com", merchant_id),
                shop_name: Some("Test Merchant".to_string()),
                shop_currency: None,
                timezone: None,
            },
        )
        .await
        .expect("Failed to create test merchant");

        // Create order directly using the function
        create_order(
            &db,
            CreateOrderRequest {
                merchant_id,
                shopify_order_id: 123456,
                name: Some("Test Order".to_string()),
                processed_at: Some(
                    chrono::DateTime::parse_from_rfc3339("2024-01-01T12:00:00Z")
                        .unwrap()
                        .with_timezone(&chrono::Utc),
                ),
                currency: Some("USD".to_string()),
                subtotal_price: Some(Decimal::from_str("100.00").unwrap()),
                total_price: Some(Decimal::from_str("110.00").unwrap()),
                total_discounts: Some(Decimal::from_str("0.00").unwrap()),
                total_shipping_price_set_amount: Some(Decimal::from_str("10.00").unwrap()),
                total_tax: Some(Decimal::from_str("0.00").unwrap()),
                financial_status: Some("paid".to_string()),
            },
        )
        .await
        .expect("Failed to create test order");

        let revenue = get_total_revenue(&db, merchant_id, None, None)
            .await
            .unwrap();

        println!("Total revenue for merchant {}: {}", merchant_id, revenue);
        assert_eq!(revenue, Decimal::from_str("110.00").unwrap());
    }

    /* #[tokio::test]
    async fn test_get_total_courier_cost() {
        let db = setup_test_db().await.expect(
            "Failed to setup test database. Make sure PostgreSQL is running and accessible.",
        );
        let merchant_id = Uuid::new_v4();

        let courier_cost = get_total_courier_cost(&db, merchant_id, None, None)
            .await
            .unwrap();
        assert_eq!(courier_cost, Decimal::ZERO);
    } */
}
