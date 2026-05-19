use anyhow::{Context, Result};
use argon2::password_hash::{rand_core::OsRng, PasswordHasher, SaltString};
use argon2::Argon2;
use clap::{Parser, Subcommand};
use std::io::{self, Write};

#[derive(Parser)]
#[command(name = "plutus", version, about = "plutus investment data store")]
struct Cli {
    #[command(subcommand)]
    command: Command,
}

#[derive(Subcommand)]
enum Command {
    /// Run the HTTP server.
    Serve,
    /// Create or update the database schema (idempotent in Phase 0).
    Migrate,
    /// Read a password from stdin and print its argon2id hash.
    HashPassword,
}

#[tokio::main]
async fn main() -> Result<()> {
    let _ = dotenvy::dotenv();
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| tracing_subscriber::EnvFilter::new("info,plutus=debug")),
        )
        .init();

    let cli = Cli::parse();
    match cli.command {
        Command::Serve => serve().await,
        Command::Migrate => migrate().await,
        Command::HashPassword => hash_password(),
    }
}

async fn migrate() -> Result<()> {
    let url = std::env::var("DATABASE_URL").context("DATABASE_URL must be set")?;
    tracing::info!("connecting to database");
    let db = plutus_storage::Db::connect(&url)
        .await
        .context("failed to connect to database")?;
    tracing::info!("applying toasty schema");
    if let Err(e) = db.migrate().await {
        // toasty 0.6's push_schema isn't idempotent — it errors on the first
        // CREATE TABLE that hits an existing table. For now we treat that as
        // expected on second+ runs and continue. Real schema diffs will need
        // a proper migration story when we hit them.
        tracing::warn!("toasty push_schema returned an error (likely tables already exist): {e}");
    }
    tracing::info!("applying post-migrate SQL");
    plutus_storage::db::post_migrate(&url)
        .await
        .context("post-migrate SQL failed")?;
    tracing::info!("seeding reference data");
    plutus_storage::seed::run(&db).await.context("seed failed")?;
    tracing::info!("migrate complete");
    Ok(())
}

async fn serve() -> Result<()> {
    let url = std::env::var("DATABASE_URL").context("DATABASE_URL must be set")?;
    let bind = std::env::var("PLUTUS_BIND_ADDR").unwrap_or_else(|_| "127.0.0.1:8080".into());
    let require_auth = std::env::var("PLUTUS_API_REQUIRE_AUTH")
        .ok()
        .map(|v| v == "true" || v == "1")
        .unwrap_or(false);
    let master_password_hash = std::env::var("PLUTUS_MASTER_PASSWORD_HASH").unwrap_or_default();
    let cookie_secret = std::env::var("PLUTUS_COOKIE_SECRET")
        .map(|s| s.into_bytes())
        .unwrap_or_else(|_| {
            tracing::warn!("PLUTUS_COOKIE_SECRET unset; using a random per-process key");
            (0..32)
                .map(|_| rand::random::<u8>())
                .collect::<Vec<_>>()
        });

    tracing::info!("connecting to database");
    let db = plutus_storage::Db::connect(&url).await?;

    let state = plutus_api::AppState {
        db,
        require_auth,
        master_password_hash,
        cookie_secret,
    };

    let app = plutus_api::build_router(state);
    let listener = tokio::net::TcpListener::bind(&bind)
        .await
        .with_context(|| format!("failed to bind {bind}"))?;
    tracing::info!("listening on http://{bind}");
    axum::serve(listener, app).await?;
    Ok(())
}

fn hash_password() -> Result<()> {
    print!("Password: ");
    io::stdout().flush()?;
    let mut password = String::new();
    io::stdin().read_line(&mut password)?;
    let password = password.trim();
    if password.is_empty() {
        anyhow::bail!("empty password");
    }
    let salt = SaltString::generate(&mut OsRng);
    let hash = Argon2::default()
        .hash_password(password.as_bytes(), &salt)
        .map_err(|e| anyhow::anyhow!("argon2: {e}"))?
        .to_string();
    println!();
    println!("Add the following to your .env (mind the quoting):");
    println!("PLUTUS_MASTER_PASSWORD_HASH={hash}");
    Ok(())
}
