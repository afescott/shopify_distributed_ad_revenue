use super::{ApiContext, AppResult};
use crate::http::types::AppError;
use axum::{extract::Query, routing::post, Extension, Json, Router};
use chrono::{DateTime, Utc};
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

mod ad_campaign;
mod courier;
mod shopify_client;

pub enum CostType {
    AdCampaign,
    CourierService,
    ShopifyIntegration,
}

pub trait Cost {
    fn insert_cost_record(&self, amount: f64, cost_type: CostType) -> anyhow::Result<()>;
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
    // 1. Get Shopify revenue (sum of total_price from orders)
    let shopify_revenue: Decimal = match (params.start_date, params.end_date) {
        (Some(start), Some(end)) => {
            sqlx::query_scalar(
                "SELECT COALESCE(SUM(total_price), 0) FROM orders 
                 WHERE merchant_id = $1 AND processed_at >= $2 AND processed_at <= $3",
            )
            .bind(params.merchant_id)
            .bind(start)
            .bind(end)
            .fetch_one(&ctx.db)
            .await?
        }
        (Some(start), None) => {
            sqlx::query_scalar(
                "SELECT COALESCE(SUM(total_price), 0) FROM orders 
                 WHERE merchant_id = $1 AND processed_at >= $2",
            )
            .bind(params.merchant_id)
            .bind(start)
            .fetch_one(&ctx.db)
            .await?
        }
        (None, Some(end)) => {
            sqlx::query_scalar(
                "SELECT COALESCE(SUM(total_price), 0) FROM orders 
                 WHERE merchant_id = $1 AND processed_at <= $2",
            )
            .bind(params.merchant_id)
            .bind(end)
            .fetch_one(&ctx.db)
            .await?
        }
        (None, None) => {
            sqlx::query_scalar(
                "SELECT COALESCE(SUM(total_price), 0) FROM orders WHERE merchant_id = $1",
            )
            .bind(params.merchant_id)
            .fetch_one(&ctx.db)
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
    let courier_cost: Decimal = match (params.start_date, params.end_date) {
        (Some(start), Some(end)) => {
            sqlx::query_scalar(
                "SELECT COALESCE(SUM(total_shipping_price_set_amount), 0) FROM orders 
                 WHERE merchant_id = $1 AND processed_at >= $2 AND processed_at <= $3",
            )
            .bind(params.merchant_id)
            .bind(start)
            .bind(end)
            .fetch_one(&ctx.db)
            .await?
        }
        (Some(start), None) => {
            sqlx::query_scalar(
                "SELECT COALESCE(SUM(total_shipping_price_set_amount), 0) FROM orders 
                 WHERE merchant_id = $1 AND processed_at >= $2",
            )
            .bind(params.merchant_id)
            .bind(start)
            .fetch_one(&ctx.db)
            .await?
        }
        (None, Some(end)) => {
            sqlx::query_scalar(
                "SELECT COALESCE(SUM(total_shipping_price_set_amount), 0) FROM orders 
                 WHERE merchant_id = $1 AND processed_at <= $2",
            )
            .bind(params.merchant_id)
            .bind(end)
            .fetch_one(&ctx.db)
            .await?
        }
        (None, None) => {
            sqlx::query_scalar(
                "SELECT COALESCE(SUM(total_shipping_price_set_amount), 0) FROM orders 
                 WHERE merchant_id = $1",
            )
            .bind(params.merchant_id)
            .fetch_one(&ctx.db)
            .await?
        }
    };

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

pub fn cost_router() -> Router {
    Router::new().route("/calculate", post(post_calculate))
}
