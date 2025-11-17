## 001_core_tenancy.sql – Columns and Responsibilities

merchants.id: UUID primary key.
merchants.shop_domain: Store’s unique domain (e.g., myshop.myshopify.com).
merchants.shop_name: Human-readable store name.
merchants.shop_currency: Store currency code (e.g., USD, GBP).
merchants.timezone: Store timezone identifier.
merchants.created_at: Timestamp when the record was created.
merchants.updated_at: Timestamp when the record was last updated.
merchants.deleted_at: Soft-delete timestamp (null means active).

shopify_installs.id: UUID primary key.
shopify_installs.merchant_id: References merchants.id (which store this install belongs to).
shopify_installs.access_scopes: OAuth scopes granted by the store.
shopify_installs.installed_at: When the app was installed.
shopify_installs.uninstalled_at: When the app was uninstalled (if ever).
shopify_installs.status: Current install status (active|uninstalled).
shopify_installs.created_at: Timestamp when the record was created.
shopify_installs.updated_at: Timestamp when the record was last updated.

app_settings.id: UUID primary key.
app_settings.merchant_id: References merchants.id (which store the settings apply to).
app_settings.revenue_basis: Revenue basis (subtotal|total).
app_settings.include_taxes: Whether to include taxes in calculations.
app_settings.include_shipping: Whether to include shipping in calculations.
app_settings.default_currency: Preferred currency override (optional).
app_settings.multi_currency_mode: Behavior for multi-currency (warn|convert).
app_settings.sync_lookback_days: How many days back to sync on initial load.
app_settings.auto_refresh_cron: Optional schedule to auto-refresh data.
app_settings.created_at: Timestamp when the record was created.
app_settings.updated_at: Timestamp when the record was last updated.

