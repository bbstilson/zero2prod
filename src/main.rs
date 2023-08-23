use std::net::TcpListener;

use anyhow::{Ok, Result};
use sqlx::postgres::PgPoolOptions;
use zero2prod::{
    configuration::get_configuration,
    startup::run,
    telemetry::{get_subscriber, init_subscriber},
};

#[tokio::main]
async fn main() -> Result<()> {
    init_subscriber(get_subscriber(
        "zero2prod".into(),
        "info".into(),
        std::io::stdout,
    ));

    let configuration = get_configuration()?;
    let connection_pool = PgPoolOptions::new()
        .acquire_timeout(std::time::Duration::from_secs(2))
        .connect_lazy_with(configuration.database.with_db());

    let listener = TcpListener::bind(configuration.application.address())?;

    run(listener, connection_pool)?.await?;

    Ok(())
}
