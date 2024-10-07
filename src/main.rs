use std::net::{Ipv4Addr, SocketAddr};

use anyhow::Result;
use axum::{extract::State, http::StatusCode, routing::get, Router};
use sqlx::{postgres::PgConnectOptions, PgPool};
use tokio::net::TcpListener;

struct DatabaseConfig {
    pub host: String,
    pub port: u16,
    pub username: String,
    pub password: String,
    pub database: String,
}

impl From<DatabaseConfig> for PgConnectOptions {
    fn from(config: DatabaseConfig) -> Self {
        Self::new()
            .host(&config.host)
            .port(config.port)
            .username(&config.username)
            .password(&config.password)
            .database(&config.database)
    }
}

fn connect_database_with(config: DatabaseConfig) -> PgPool {
    let options: PgConnectOptions = config.into();
    PgPool::connect_lazy_with(options)
}

async fn health_check_db(State(db): State<PgPool>) -> StatusCode {
    let row = sqlx::query("SELECT 1")
        .fetch_one(&db)
        .await;
    match row {
        Ok(_) => StatusCode::OK,
        Err(_) => StatusCode::INTERNAL_SERVER_ERROR,
    }
}

pub async fn health_check() -> StatusCode {
    StatusCode::OK
}

#[tokio::main]
async fn main() -> Result<()> {
    let database_config = DatabaseConfig {
        host: "localhost".to_string(),
        port: 5432,
        username: "app".to_string(),
        password: "passwd".to_string(),
        database: "app".to_string(),
    };
    let pool = connect_database_with(database_config);


    let app  = Router::new()
    .route("/health", get(health_check))
    .route("/health_db", get(health_check_db))
    .with_state(pool);

    let addr = SocketAddr::new(Ipv4Addr::LOCALHOST.into(), 8080);
    let listener = TcpListener::bind(addr).await?;

    println!("Listening on {}", addr);

    Ok(axum::serve(listener, app).await?)
}

#[tokio::test]
async fn test_health_check() {
    let response = health_check().await;
    assert_eq!(response, StatusCode::OK);
}

#[sqlx::test]
async fn test_health_check_db(pool: PgPool) {
    let response = health_check_db(State(pool)).await;
    assert_eq!(response, StatusCode::OK);
}
