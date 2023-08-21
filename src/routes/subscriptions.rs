use actix_web::{web, HttpResponse};
use serde::Deserialize;
use sqlx::PgPool;
use uuid::Uuid;

#[derive(Deserialize)]
pub struct FormData {
    email: String,
    name: String,
}

pub async fn subscribe(form: web::Form<FormData>, pool: web::Data<PgPool>) -> HttpResponse {
    let r = sqlx::query!(
        r#"
        insert into subscriptions (id, email, name) values
        ($1, $2, $3)
        "#,
        Uuid::new_v4(),
        form.email,
        form.name,
    )
    .execute(pool.get_ref())
    .await;

    match r {
        Ok(_) => HttpResponse::Ok().finish(),
        Err(e) => {
            println!("Failed to execute query: {}", e);
            HttpResponse::InternalServerError().finish()
        }
    }
}
