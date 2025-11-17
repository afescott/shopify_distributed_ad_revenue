use serde::{Deserialize, Serialize};

// Shopify API Response Types

#[derive(Debug, Deserialize)]
pub struct ShopifyApiResponse<T> {
    pub products: Option<Vec<T>>,
    pub orders: Option<Vec<T>>,
}

// Product Types
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ShopifyProduct {
    pub id: i64,
    pub title: String,
    pub body_html: Option<String>,
    pub vendor: Option<String>,
    pub product_type: Option<String>,
    pub created_at: String,
    pub updated_at: String,
    pub status: Option<String>,
    pub variants: Vec<ShopifyVariant>,
    pub images: Vec<ShopifyProductImage>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ShopifyVariant {
    pub id: i64,
    pub product_id: i64,
    pub title: String,
    pub price: String,
    pub sku: Option<String>,
    pub barcode: Option<String>,
    pub weight: Option<f64>,
    pub weight_unit: Option<String>,
    pub inventory_quantity: Option<i32>,
    pub inventory_item_id: Option<i64>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ShopifyProductImage {
    pub id: i64,
    pub product_id: i64,
    pub src: String,
    pub alt: Option<String>,
}

// Order Types
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ShopifyOrder {
    pub id: i64,
    pub name: String,
    pub email: Option<String>,
    pub created_at: String,
    pub updated_at: String,
    pub processed_at: Option<String>,
    pub currency: String,
    pub subtotal_price: String,
    pub total_price: String,
    pub total_discounts: String,
    pub total_shipping_price_set: ShopifyPriceSet,
    pub total_tax: String,
    pub financial_status: Option<String>,
    pub fulfillment_status: Option<String>,
    pub cancelled_at: Option<String>,
    pub line_items: Vec<ShopifyLineItem>,
    pub customer: Option<ShopifyCustomer>,
    pub shipping_address: Option<ShopifyAddress>,
    pub billing_address: Option<ShopifyAddress>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ShopifyPriceSet {
    pub shop_money: ShopifyMoney,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ShopifyMoney {
    pub amount: String,
    pub currency_code: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ShopifyLineItem {
    pub id: i64,
    pub product_id: Option<i64>,
    pub variant_id: Option<i64>,
    pub title: String,
    pub quantity: i32,
    pub price: String,
    pub sku: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ShopifyCustomer {
    pub id: i64,
    pub email: Option<String>,
    pub first_name: Option<String>,
    pub last_name: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ShopifyAddress {
    pub first_name: Option<String>,
    pub last_name: Option<String>,
    pub address1: Option<String>,
    pub city: Option<String>,
    pub province: Option<String>,
    pub country: Option<String>,
    pub zip: Option<String>,
}

// API Error Types
#[derive(Debug, Deserialize)]
pub struct ShopifyError {
    pub errors: serde_json::Value,
}

#[derive(Debug, thiserror::Error)]
pub enum ShopifyErrorType {
    #[error("HTTP error: {0}")]
    Http(#[from] reqwest::Error),
    #[error("API error: {0}")]
    Api(String),
    #[error("Authentication failed")]
    Authentication,
    #[error("Rate limit exceeded")]
    RateLimit,
}


