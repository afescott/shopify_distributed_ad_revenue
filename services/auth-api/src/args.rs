use clap::Parser;
use serde::{Deserialize, Serialize};

#[derive(Parser, Debug, Clone)]
#[command(author, version, about, long_about = None)]
pub struct CliArgs {
    #[arg(long, env = "JWT_PRIVATE_KEY")]
    pub private_key: Option<String>,

    #[arg(long, env = "JWT_PUBLIC_KEY")]
    pub public_key: Option<String>,
    /// PostgreSQL database URL
    #[arg(long, env = "DATABASE_URL")]
    pub database_url: Option<String>,

    /// SMTP server host
    #[arg(long, env = "SMTP_HOST")]
    pub smtp_host: Option<String>,

    /// SMTP server port
    #[arg(long, env = "SMTP_PORT")]
    pub smtp_port: Option<u16>,

    /// SMTP username
    #[arg(long, env = "SMTP_USERNAME")]
    pub smtp_username: Option<String>,

    /// SMTP password
    #[arg(long, env = "SMTP_PASSWORD")]
    pub smtp_password: Option<String>,

    /// SMTP from email
    #[arg(long, env = "SMTP_FROM_EMAIL")]
    pub smtp_from_email: Option<String>,

    /// Enable email sending
    #[arg(long, env = "ENABLE_EMAIL")]
    pub enable_email: Option<bool>,

    /// JWT expiration hours
    #[arg(long, env = "JWT_EXPIRATION_HOURS")]
    pub jwt_expiration_hours: Option<u64>,

    /// Server base URL (for email links and SMTP configuration)
    #[arg(long, env = "DARKEX_URL")]
    pub darkex_url: Option<String>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Args {
    pub private_key: Option<String>,
    pub public_key: Option<String>,
    pub database_url: String,
    pub smtp_host: Option<String>,
    pub smtp_port: Option<u16>,
    pub smtp_username: Option<String>,
    pub smtp_password: Option<String>,
    pub smtp_from_email: String,
    pub enable_email: bool,
    pub jwt_expiration_hours: u64,
    pub darkex_url: String,
}

impl Default for Args {
    fn default() -> Self {
        Self {
            private_key: None,
            public_key: None,
            database_url: "postgres://exchange_user:exchange_password@localhost/exchange_api"
                .to_string(),
            smtp_host: Some("smtp.gmail.com".to_string()),
            smtp_port: Some(587),
            smtp_username: Some("test.darkex2025@gmail.com".to_string()),
            smtp_password: Some("bmzs vrej jbyr nbut".to_string()),
            smtp_from_email: "test.darkex2025@gmail.com".to_string(),
            enable_email: true,
            jwt_expiration_hours: 24,
            darkex_url: "http://localhost:8080".to_string(),
        }
    }
}

impl From<CliArgs> for Args {
    fn from(cli_args: CliArgs) -> Self {
        let default = Args::default();

        Self {
            private_key: cli_args.private_key,
            public_key: cli_args.public_key,
            database_url: cli_args.database_url.unwrap_or(default.database_url),
            smtp_host: cli_args.smtp_host.or(default.smtp_host),
            smtp_port: cli_args.smtp_port.or(default.smtp_port),
            smtp_username: cli_args.smtp_username.or(default.smtp_username),
            smtp_password: cli_args.smtp_password.or(default.smtp_password),
            smtp_from_email: cli_args.smtp_from_email.unwrap_or(default.smtp_from_email),
            enable_email: cli_args.enable_email.unwrap_or(default.enable_email),
            jwt_expiration_hours: cli_args
                .jwt_expiration_hours
                .unwrap_or(default.jwt_expiration_hours),
            darkex_url: cli_args.darkex_url.unwrap_or(default.darkex_url),
        }
    }
}
