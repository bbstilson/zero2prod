use std::time::Duration;

use sqlx::{PgPool, Postgres, Transaction};
use tracing::{field::display, Span};
use uuid::Uuid;

use crate::{
    configuration::Settings, domain::SubscriberEmail, email_client::EmailClient,
    startup::get_connection_pool,
};

type PgTx = Transaction<'static, Postgres>;

#[tracing::instrument(skip_all, fields(
    newsletter_issue_id=tracing::field::Empty,
    subscriber_email=tracing::field::Empty,
))]
pub async fn try_execute_task(
    pool: &PgPool,
    email_client: &EmailClient,
) -> Result<ExecutionOutcome, anyhow::Error> {
    let task = dequeue_task(pool).await?;
    if task.is_none() {
        return Ok(ExecutionOutcome::EmptyQueue);
    }
    let (tx, issue_id, email) = task.unwrap();

    Span::current()
        .record("newsletter_issue_id", &display(issue_id))
        .record("subscriber_email", &display(&email));

    match SubscriberEmail::parse(email.clone()) {
        Ok(email) => {
            let issue = get_issue(pool, issue_id).await?;

            if let Err(e) = email_client
                .send_email(
                    &email,
                    &issue.title,
                    &issue.html_content,
                    &issue.text_content,
                )
                .await
            {
                tracing::error!(
                    error.cause_chain = ?e,
                    error.message = %e,
                    "Failed to deliver issue to a confirmed subscriber. Skipping"
                )
            }
        }
        Err(e) => {
            tracing::error!(
                error.cause_chain = ?e,
                error.message = %e,
                "Skipping a confirmed subscriber. \
                Their stored contact details are invalid"
            );
        }
    }
    delete_task(tx, issue_id, &email).await?;

    Ok(ExecutionOutcome::TaskCompleted)
}

#[tracing::instrument(skip_all)]
async fn dequeue_task(pool: &PgPool) -> Result<Option<(PgTx, Uuid, String)>, anyhow::Error> {
    let mut tx = pool.begin().await?;

    let r = sqlx::query!(
        r#"
        select newsletter_issue_id, subscriber_email
        from issue_delivery_queue
        for update
        skip locked
        limit 1
        "#,
    )
    .fetch_optional(tx.as_mut())
    .await?;

    if let Some(r) = r {
        Ok(Some((tx, r.newsletter_issue_id, r.subscriber_email)))
    } else {
        Ok(None)
    }
}

#[tracing::instrument(skip_all)]
async fn delete_task(mut tx: PgTx, issue_id: Uuid, email: &str) -> Result<(), anyhow::Error> {
    sqlx::query!(
        r#"
        delete from issue_delivery_queue
        where newsletter_issue_id = $1
        and subscriber_email = $2
        "#,
        issue_id,
        email
    )
    .execute(tx.as_mut())
    .await?;

    tx.commit().await?;

    Ok(())
}

struct NewsletterIssue {
    title: String,
    text_content: String,
    html_content: String,
}

#[tracing::instrument(skip_all)]
async fn get_issue(pool: &PgPool, issue_id: Uuid) -> Result<NewsletterIssue, anyhow::Error> {
    let issue = sqlx::query_as!(
        NewsletterIssue,
        r#"
        select title, text_content, html_content
        from newsletter_issues
        where newsletter_issue_id = $1
        "#,
        issue_id
    )
    .fetch_one(pool)
    .await?;

    Ok(issue)
}

pub enum ExecutionOutcome {
    TaskCompleted,
    EmptyQueue,
}

#[tracing::instrument(skip_all)]
async fn worker_loop(pool: &PgPool, email_client: &EmailClient) -> Result<(), anyhow::Error> {
    loop {
        match try_execute_task(pool, email_client).await {
            Ok(ExecutionOutcome::TaskCompleted) => {}
            Ok(ExecutionOutcome::EmptyQueue) => {
                tokio::time::sleep(Duration::from_secs(10)).await;
            }
            Err(_) => {
                tokio::time::sleep(Duration::from_secs(10)).await;
            }
        }
    }
}

pub async fn run_worker_until_stopped(configuration: Settings) -> Result<(), anyhow::Error> {
    let pool = get_connection_pool(&configuration.database);
    let email_client = configuration.email.client();

    worker_loop(&pool, &email_client).await
}
