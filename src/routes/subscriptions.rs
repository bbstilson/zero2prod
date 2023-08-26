use actix_web::{web, HttpResponse};
use anyhow::Result;
use serde::Deserialize;
use sqlx::PgPool;
use uuid::Uuid;

use crate::domain::{NewSubscriber, SubscriberEmail, SubscriberName};

#[derive(Deserialize)]
pub struct FormData {
    email: String,
    name: String,
}

impl TryFrom<FormData> for NewSubscriber {
    type Error = ();

    fn try_from(f: FormData) -> Result<Self, Self::Error> {
        let name = SubscriberName::parse(f.name)?;
        let email = SubscriberEmail::parse(f.email)?;
        Ok(Self { email, name })
    }
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

    let new_subscriber = match form.0.try_into() {
        Ok(form) => form,
        Err(_) => return HttpResponse::BadRequest().finish(),
    };

    match insert_subscriber(&pool, &new_subscriber).await {
        Ok(_) => HttpResponse::Ok().finish(),
        Err(_) => HttpResponse::InternalServerError().finish(),
    }
}

#[allow(clippy::async_yields_async)]
#[tracing::instrument(skip(new_subscriber, pool))]
pub async fn insert_subscriber(pool: &PgPool, new_subscriber: &NewSubscriber) -> Result<()> {
    sqlx::query!(
        r#"
        insert into subscriptions (id, email, name) values
        ($1, $2, $3)
        "#,
        Uuid::new_v4(),
        new_subscriber.email.as_ref(),
        new_subscriber.name.as_ref()
    )
    .execute(pool)
    .await
    .map_err(|e| {
        tracing::error!("Failed to execute query: {:?}", e);
        e
    })?;

    Ok(())
}
