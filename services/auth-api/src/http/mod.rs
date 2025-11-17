use std::sync::Arc;

use anyhow::Context;
use axum::{response::Redirect, routing::get, Extension, Router};
/* use sqlx::prelude::FromRow; */
use sqlx::PgPool;
use tower_http::{cors::CorsLayer, services::ServeDir, trace::TraceLayer};

use crate::auth::jkws::AuthService;
use crate::Args;

mod auth;
mod inventory;
mod orders;
mod products;
mod types;
mod users;

pub use types::*;

#[derive(Clone)]
pub struct ApiContext {
    pub config: Arc<Args>,
    pub db: PgPool,
    pub auth_service: Arc<AuthService>,
}

pub async fn serve(config: Args, db: PgPool) -> anyhow::Result<()> {
    let auth_service = Arc::new(AuthService::from_config(&config)?);

    // Initialize auxiliary services here (email, etc.) when available

    let app = api_router()
        .layer(Extension(ApiContext {
            config: Arc::new(config),
            db,
            auth_service: auth_service.clone(),
        }))
        // Enable CORS for cross-origin requests (needed for Swagger UI)
        .layer(
            CorsLayer::new()
                .allow_origin(tower_http::cors::Any)
                .allow_methods([
                    axum::http::Method::GET,
                    axum::http::Method::POST,
                    axum::http::Method::PUT,
                    axum::http::Method::DELETE,
                ])
                .allow_headers([
                    axum::http::header::CONTENT_TYPE,
                    axum::http::header::AUTHORIZATION,
                ]),
        )
        // Enables logging. Use `RUST_LOG=tower_http=debug`
        .layer(TraceLayer::new_for_http());

    let listener = tokio::net::TcpListener::bind("0.0.0.0:8080")
        .await
        .context("could not bind to")?;

    axum::serve(listener, app.into_make_service())
        .await
        .context("error running HTTP server")
}

fn api_router() -> Router {
    // This is the order that the modules were authored in.
    Router::new()
        // Redirect root to docs
        .route("/", get(|| async { Redirect::permanent("/docs/") }))
        // Serve static documentation files
        .nest_service("/docs", ServeDir::new("docs"))
        // API routes
        .nest(
            "/api/v1",
            Router::new()
                .merge(auth::auth_router())
                .merge(inventory::inventory_router())
                .merge(orders::orders_router())
                .merge(products::products_router())
                .merge(users::users_router()),
        )
}
