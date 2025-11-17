use crate::shopify::types::*;
use reqwest::Client;
use std::time::Duration;

/// Shopify Admin API Client
///
/// This client handles authentication and API calls to Shopify Admin API
/// to retrieve products and orders.
pub struct ShopifyClient {
    store_name: String,
    access_token: String,
    api_version: String,
    client: Client,
}

impl ShopifyClient {
    /// Create a new Shopify API client
    ///
    /// # Arguments
    /// * `store_name` - Your Shopify store name (e.g., "store-analytic-app")
    /// * `access_token` - Your Shopify Admin API access token
    /// * `api_version` - API version (e.g., "2024-10" or "2025-01")
    pub fn new(store_name: String, access_token: String, api_version: String) -> Self {
        let client = Client::builder()
            .timeout(Duration::from_secs(30))
            .build()
            .expect("Failed to create HTTP client");

        Self {
            store_name,
            access_token,
            api_version,
            client,
        }
    }

    /// Build the base URL for API requests
    fn base_url(&self) -> String {
        format!(
            "https://{}.myshopify.com/admin/api/{}",
            self.store_name, self.api_version
        )
    }

    /// Build headers for authenticated requests
    fn headers(&self) -> reqwest::header::HeaderMap {
        let mut headers = reqwest::header::HeaderMap::new();
        headers.insert("X-Shopify-Access-Token", self.access_token.parse().unwrap());
        headers.insert(
            reqwest::header::CONTENT_TYPE,
            "application/json".parse().unwrap(),
        );
        headers
    }

    /// Fetch products from Shopify
    ///
    /// # Arguments
    /// * `limit` - Maximum number of products to fetch per page (default: 250, max: 250)
    /// * `since_id` - Fetch products with ID greater than this value (for pagination)
    ///
    /// # Returns
    /// Vector of ShopifyProduct objects
    pub async fn get_products(
        &self,
        limit: Option<u32>,
        since_id: Option<i64>,
    ) -> Result<Vec<ShopifyProduct>, ShopifyErrorType> {
        let limit = limit.unwrap_or(250).min(250);
        let url = format!("{}/products.json", self.base_url());

        let mut query_params = vec![("limit", limit.to_string())];
        if let Some(id) = since_id {
            query_params.push(("since_id", id.to_string()));
        }

        let response = self
            .client
            .get(&url)
            .headers(self.headers())
            .query(&query_params)
            .send()
            .await?;

        self.handle_response(response).await
    }

    /// Fetch a single product by ID
    pub async fn get_product(&self, product_id: i64) -> Result<ShopifyProduct, ShopifyErrorType> {
        let url = format!("{}/products/{}.json", self.base_url(), product_id);

        let response = self.client.get(&url).headers(self.headers()).send().await?;

        let mut wrapper: serde_json::Value = response.json().await?;
        let product = serde_json::from_value(wrapper["product"].take())
            .map_err(|e| ShopifyErrorType::Api(format!("Failed to parse product: {}", e)))?;

        Ok(product)
    }

    /// Fetch orders from Shopify
    ///
    /// # Arguments
    /// * `limit` - Maximum number of orders to fetch per page (default: 250, max: 250)
    /// * `since_id` - Fetch orders with ID greater than this value (for pagination)
    /// * `status` - Filter by order status: "any", "open", "closed", "cancelled"
    /// * `financial_status` - Filter by financial status: "any", "authorized", "pending", "paid", "refunded", etc.
    ///
    /// # Returns
    /// Vector of ShopifyOrder objects
    pub async fn get_orders(
        &self,
        limit: Option<u32>,
        since_id: Option<i64>,
        status: Option<&str>,
        financial_status: Option<&str>,
    ) -> Result<Vec<ShopifyOrder>, ShopifyErrorType> {
        let limit = limit.unwrap_or(250).min(250);
        let url = format!("{}/orders.json", self.base_url());

        let mut query_params = vec![("limit", limit.to_string())];
        if let Some(id) = since_id {
            query_params.push(("since_id", id.to_string()));
        }
        if let Some(s) = status {
            query_params.push(("status", s.to_string()));
        }
        if let Some(fs) = financial_status {
            query_params.push(("financial_status", fs.to_string()));
        }

        let response = self
            .client
            .get(&url)
            .headers(self.headers())
            .query(&query_params)
            .send()
            .await?;

        self.handle_response(response).await
    }

    /// Fetch a single order by ID
    pub async fn get_order(&self, order_id: i64) -> Result<ShopifyOrder, ShopifyErrorType> {
        let url = format!("{}/orders/{}.json", self.base_url(), order_id);

        let response = self.client.get(&url).headers(self.headers()).send().await?;

        let mut wrapper: serde_json::Value = response.json().await?;
        let order = serde_json::from_value(wrapper["order"].take())
            .map_err(|e| ShopifyErrorType::Api(format!("Failed to parse order: {}", e)))?;

        Ok(order)
    }

    /// Handle API response and check for errors
    async fn handle_response<T>(&self, response: reqwest::Response) -> Result<T, ShopifyErrorType>
    where
        T: serde::de::DeserializeOwned,
    {
        let status = response.status();

        // Check rate limiting
        if status == 429 {
            return Err(ShopifyErrorType::RateLimit);
        }

        // Check authentication
        if status == 401 {
            return Err(ShopifyErrorType::Authentication);
        }

        // Parse response body
        let text = response.text().await?;

        if !status.is_success() {
            // Try to parse error response
            if let Ok(error) = serde_json::from_str::<ShopifyError>(&text) {
                return Err(ShopifyErrorType::Api(format!(
                    "Shopify API error: {}",
                    serde_json::to_string(&error.errors).unwrap_or_default()
                )));
            }
            return Err(ShopifyErrorType::Api(format!("HTTP {}: {}", status, text)));
        }

        // Parse successful response
        let json: serde_json::Value = serde_json::from_str(&text)
            .map_err(|e| ShopifyErrorType::Api(format!("Invalid JSON: {}", e)))?;

        // Handle both {products: [...]} and {orders: [...]} formats
        let data = if json.get("products").is_some() {
            json["products"].clone()
        } else if json.get("orders").is_some() {
            json["orders"].clone()
        } else {
            json
        };

        serde_json::from_value(data)
            .map_err(|e| ShopifyErrorType::Api(format!("Failed to parse response: {}", e)))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_base_url() {
        let client = ShopifyClient::new(
            "test-store".to_string(),
            "token".to_string(),
            "2024-10".to_string(),
        );
        assert_eq!(
            client.base_url(),
            "https://test-store.myshopify.com/admin/api/2024-10"
        );
    }
}

