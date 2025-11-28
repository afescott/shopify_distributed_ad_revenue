use anyhow::Context;
use chrono::{DateTime, Utc};
use rust_decimal::Decimal;
use sqlx::PgPool;
use uuid::Uuid;

pub struct CostEngineConfig {
    pub database_url: String,
    pub api_keys: Vec<String>,
    pub mapping_table_path: String,
    pub manual_data_path: String,
}

pub struct CostEngineProducer {
    config: CostEngineConfig,
}

#[derive(Debug)]
pub struct ProfitCalculation {
    pub shopify_revenue: Decimal,
    pub shopify_product_cost: Decimal,
    pub ad_cost: Decimal,
    pub courier_cost: Decimal,
    pub manual_cost: Decimal,
    pub profit: Decimal,
}

/// Calculate profit by retrieving data from SQL
/// 
/// # Arguments
/// * `db` - PostgreSQL connection pool
/// * `merchant_id` - UUID of the merchant to calculate profit for
/// * `start_date` - Optional start date for the calculation period
/// * `end_date` - Optional end date for the calculation period
pub async fn post_calculate(
    db: &PgPool,
    merchant_id: Uuid,
    start_date: Option<DateTime<Utc>>,
    end_date: Option<DateTime<Utc>>,
) -> anyhow::Result<ProfitCalculation> {
    // 1. Get Shopify revenue (sum of total_price from orders)
    let shopify_revenue: Decimal = match (start_date, end_date) {
        (Some(start), Some(end)) => {
            sqlx::query_scalar(
                "SELECT COALESCE(SUM(total_price), 0) FROM orders 
                 WHERE merchant_id = $1 AND processed_at >= $2 AND processed_at <= $3"
            )
            .bind(merchant_id)
            .bind(start)
            .bind(end)
            .fetch_one(db)
            .await?
        }
        (Some(start), None) => {
            sqlx::query_scalar(
                "SELECT COALESCE(SUM(total_price), 0) FROM orders 
                 WHERE merchant_id = $1 AND processed_at >= $2"
            )
            .bind(merchant_id)
            .bind(start)
            .fetch_one(db)
            .await?
        }
        (None, Some(end)) => {
            sqlx::query_scalar(
                "SELECT COALESCE(SUM(total_price), 0) FROM orders 
                 WHERE merchant_id = $1 AND processed_at <= $2"
            )
            .bind(merchant_id)
            .bind(end)
            .fetch_one(db)
            .await?
        }
        (None, None) => {
            sqlx::query_scalar(
                "SELECT COALESCE(SUM(total_price), 0) FROM orders WHERE merchant_id = $1"
            )
            .bind(merchant_id)
            .fetch_one(db)
            .await?
        }
    };

    // 2. Get Shopify product cost (placeholder - would need product cost data)
    // TODO: Replace with actual product cost query when cost data is available
    let shopify_product_cost: Decimal = Decimal::ZERO;

    // 3. Get ad cost (placeholder query - would need ad_cost table)
    // TODO: Replace with actual ad_cost query when ad_cost table is available
    let ad_cost: Decimal = Decimal::ZERO;

    // 4. Get courier cost (sum of total_shipping_price_set_amount from orders)
    let courier_cost: Decimal = match (start_date, end_date) {
        (Some(start), Some(end)) => {
            sqlx::query_scalar(
                "SELECT COALESCE(SUM(total_shipping_price_set_amount), 0) FROM orders 
                 WHERE merchant_id = $1 AND processed_at >= $2 AND processed_at <= $3"
            )
            .bind(merchant_id)
            .bind(start)
            .bind(end)
            .fetch_one(db)
            .await?
        }
        (Some(start), None) => {
            sqlx::query_scalar(
                "SELECT COALESCE(SUM(total_shipping_price_set_amount), 0) FROM orders 
                 WHERE merchant_id = $1 AND processed_at >= $2"
            )
            .bind(merchant_id)
            .bind(start)
            .fetch_one(db)
            .await?
        }
        (None, Some(end)) => {
            sqlx::query_scalar(
                "SELECT COALESCE(SUM(total_shipping_price_set_amount), 0) FROM orders 
                 WHERE merchant_id = $1 AND processed_at <= $2"
            )
            .bind(merchant_id)
            .bind(end)
            .fetch_one(db)
            .await?
        }
        (None, None) => {
            sqlx::query_scalar(
                "SELECT COALESCE(SUM(total_shipping_price_set_amount), 0) FROM orders 
                 WHERE merchant_id = $1"
            )
            .bind(merchant_id)
            .fetch_one(db)
            .await?
        }
    };

    // 5. Get manual cost (placeholder query - would need manual_cost table)
    // TODO: Replace with actual manual_cost query when manual_cost table is available
    let manual_cost: Decimal = Decimal::ZERO;

    // Calculate profit
    let profit = shopify_revenue - shopify_product_cost - ad_cost - courier_cost - manual_cost;

    Ok(ProfitCalculation {
        shopify_revenue,
        shopify_product_cost,
        ad_cost,
        courier_cost,
        manual_cost,
        profit,
    })
}
