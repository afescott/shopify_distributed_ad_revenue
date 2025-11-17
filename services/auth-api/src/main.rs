use anyhow::Context;
use args::{Args, CliArgs};
use clap::Parser;
use sqlx::postgres::PgPoolOptions;

mod args;
mod auth;
mod http;
pub mod misc;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let cli_args = CliArgs::parse();
    let config = Args::from(cli_args);

    let db = PgPoolOptions::new()
        // The default connection limit for a Postgres server is 100 connections, minus 3 for superusers.
        // Since we're using the default superuser we don't have to worry about this too much,
        // although we should leave some connections available for manual access.
        //
        // If you're deploying your application with multiple replicas, then the total
        // across all replicas should not exceed the Postgres connection limit.
        .max_connections(50)
        .connect(&config.database_url)
        .await
        .context("could not connect to database_url")?;

    sqlx::migrate!("./sql/migrations")
        .run(&db)
        .await
        .context("could not run migrations")?;

    http::serve(config, db).await?;

    Ok(())
}

