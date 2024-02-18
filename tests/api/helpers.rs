use anyhow::Result;
use once_cell::sync::Lazy;
use sqlx::{Connection, Executor, PgConnection, PgPool};
use uuid::Uuid;
use wiremock::MockServer;
use zero2prod::{
    configuration::{get_configuration, DatabaseSettings},
    startup::{get_connection_pool, Application},
    telemetry::{get_subscriber, init_subscriber},
};

static TRACING: Lazy<()> = Lazy::new(|| {
    let default_filter_level = "info".into();
    let subscriber_name = "test".into();

    if std::env::var("TEST_LOG").is_ok() {
        init_subscriber(get_subscriber(
            subscriber_name,
            default_filter_level,
            std::io::stdout,
        ));
    } else {
        init_subscriber(get_subscriber(
            subscriber_name,
            default_filter_level,
            std::io::sink,
        ));
    };
});

/// Confirmation links embedded in the request to the email API.
pub struct ConfirmationLinks {
    pub html: reqwest::Url,
    pub plain_text: reqwest::Url,
}

pub struct TestApp {
    pub port: u16,
    pub address: String,
    pub db_pool: PgPool,
    pub email_server: MockServer,
}

impl TestApp {
    pub async fn post_newsletters(&self, body: serde_json::Value) -> reqwest::Response {
        reqwest::Client::new()
            .post(&format!("{}/newsletters", &self.address))
            .basic_auth(Uuid::new_v4().to_string(), Some(Uuid::new_v4().to_string()))
            .json(&body)
            .send()
            .await
            .expect("failed to execute request")
    }

    pub async fn post_subscriptions(&self, body: String) -> reqwest::Response {
        reqwest::Client::new()
            .post(&format!("{}/subscriptions", self.address))
            .header("Content-Type", "application/x-www-form-urlencoded")
            .body(body)
            .send()
            .await
            .expect("Failed to execute request")
    }

    pub fn get_confirmation_links(&self, email_request: &wiremock::Request) -> ConfirmationLinks {
        let body: serde_json::Value = serde_json::from_slice(&email_request.body).unwrap();
        let get_link = |s: &str| {
            let links: Vec<_> = linkify::LinkFinder::new()
                .links(s)
                .filter(|l| *l.kind() == linkify::LinkKind::Url)
                .collect();
            assert_eq!(links.len(), 1);
            let raw_link = links[0].as_str().to_owned();
            let mut confirmation_link = reqwest::Url::parse(&raw_link).unwrap();
            // Let's make sure we don't call random APIs on the web.
            assert_eq!(confirmation_link.host_str().unwrap(), "127.0.0.1");
            confirmation_link.set_port(Some(self.port)).unwrap();
            confirmation_link
        };
        let html = get_link(&body["HtmlBody"].as_str().unwrap());
        let plain_text = get_link(&body["TextBody"].as_str().unwrap());

        ConfirmationLinks { html, plain_text }
    }
}

pub async fn spawn_app() -> TestApp {
    // The first time `initialize` is invoked the code in `TRACING` is executed.
    // All other invocations will instead skip execution.
    Lazy::force(&TRACING);

    let email_server = MockServer::start().await;

    let configuration = {
        let mut c = get_configuration().expect("Failed to get test configuration");
        // Use a different database for each test.
        c.database.database_name = Uuid::new_v4().to_string();
        // Use a random OS port.
        c.application.port = 0;
        // Use the mock server as email API.
        c.email.base_url = email_server.uri();
        c
    };

    configure_test_database(&configuration.database)
        .await
        .expect("Failed to configure test database");

    let application = Application::build(configuration.clone())
        .await
        .expect("Failed to build test application");
    let port = application.port();
    let address = format!("http://127.0.0.1:{}", application.port());

    let _ = tokio::spawn(application.run_until_stopped());

    TestApp {
        port,
        address,
        db_pool: get_connection_pool(&configuration.database),
        email_server,
    }
}

async fn configure_test_database(config: &DatabaseSettings) -> Result<()> {
    let mut connection = PgConnection::connect_with(&config.without_db()).await?;

    connection
        .execute(format!(r#"create database "{}""#, config.database_name).as_str())
        .await?;

    let connection_pool = PgPool::connect_with(config.with_db()).await?;

    sqlx::migrate!("./migrations").run(&connection_pool).await?;

    Ok(())
}
