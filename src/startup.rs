use std::net::TcpListener;

use actix_web::{dev::Server, web, App, HttpServer};
use anyhow::Result;
use sqlx::PgPool;
use tracing_actix_web::TracingLogger;

use crate::routes::*;

pub fn run(listener: TcpListener, db_pool: PgPool) -> Result<Server> {
    let db_pool = web::Data::new(db_pool);

    let server = HttpServer::new(move || {
        App::new()
            .wrap(TracingLogger::default())
            .route("/health_check", web::get().to(health_check))
            .route("/subscribe", web::post().to(subscribe))
            .app_data(db_pool.clone())
    })
    .listen(listener)?
    .run();

    Ok(server)
}
