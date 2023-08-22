use actix_web::{web, HttpResponse};
use anyhow::Result;
use serde::Deserialize;
use sqlx::PgPool;
use uuid::Uuid;

#[derive(Deserialize)]
pub struct FormData {
    email: String,
    name: String,
}

#[tracing::instrument(
    skip(form, pool),
    fields(
        subscriber_email = %form.email,
        subscriber_name = %form.name
    )
)]
pub async fn subscribe(form: web::Form<FormData>, pool: web::Data<PgPool>) -> HttpResponse {
    sqlx::query!("delete from subscriptions")
        .execute(pool.as_ref())
        .await
        .expect("wahh");

    match insert_subscriber(&pool, &form).await {
        Ok(_) => HttpResponse::Ok().finish(),
        Err(_) => HttpResponse::InternalServerError().finish(),
    }
}

#[allow(clippy::async_yields_async)]
#[tracing::instrument(skip(form, pool))]
pub async fn insert_subscriber(pool: &PgPool, form: &FormData) -> Result<()> {
    sqlx::query!(
        r#"
        insert into subscriptions (id, email, name) values
        ($1, $2, $3)
        "#,
        Uuid::new_v4(),
        form.email,
        form.name,
    )
    .execute(pool)
    .await
    .map_err(|e| {
        tracing::error!("Failed to execute query: {:?}", e);
        e
    })?;

    Ok(())
}
