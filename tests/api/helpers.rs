use once_cell::sync::Lazy;
use sqlx::{Connection, Executor, PgConnection, PgPool};
use std::net::TcpListener;
use uuid::Uuid;
use zero2prod::{
    configuration::{get_configuration, DatabaseSettings},
    email_client::EmailClient,
    telemetry::{get_subscriber, init_subscriber},
};

// Ensure that the `tracing` stack is only initialised once using `once_cell`
static TRACING: Lazy<()> = Lazy::new(|| {
    let default_filter_level = "debug".into();
    let subscriber_name = "test".into();

    if std::env::var("TEST_LOG").is_ok() {
        let subscriber = get_subscriber(subscriber_name, default_filter_level, std::io::stdout);
        init_subscriber(subscriber);
    } else {
        let subscriber = get_subscriber(subscriber_name, default_filter_level, std::io::sink);
        init_subscriber(subscriber);
    }
});

pub struct TestApp {
    pub address: String,
    pub db_pool: PgPool,
}

// Launch our application in the background ~somehow~
pub async fn spawn_app() -> TestApp {
    // The first time `initialize`is invoked the code in `TRACING` is executed.
    // All other invocations will instead skip execution.
    Lazy::force(&TRACING);

    let listener = TcpListener::bind("127.0.0.1:0").expect("Failed to bind random port");
    let port = listener.local_addr().unwrap().port();

    let mut config = get_configuration().expect("Failed to read configuration.");

    config.database.database_name = Uuid::new_v4().to_string();

    let conn_pool = configure_database(&config.database).await;

    let sender_email = config
        .email_client
        .sender()
        .expect("Invalid sender email address.");

    let timeout = config.email_client.timeout();
    let client = EmailClient::new(
        config.email_client.base_url,
        sender_email,
        config.email_client.authorization_token,
        timeout,
    );

    let server = zero2prod::startup::run(listener, conn_pool.clone(), client)
        .expect("Failed to bind address");

    tokio::spawn(server);

    TestApp {
        address: format!("http://127.0.0.1:{port}"),
        db_pool: conn_pool,
    }
}

async fn configure_database(config: &DatabaseSettings) -> PgPool {
    // Create database
    //
    let mut connection = PgConnection::connect_with(&config.without_db())
        .await
        .expect("Failed to connect to Postgres");

    connection
        .execute(format!(r#"CREATE DATABASE "{}";"#, config.database_name).as_str())
        .await
        .expect("Failed to create database.");

    let connection_pool = PgPool::connect_with(config.with_db())
        .await
        .expect("Failed to connect to postgres");

    sqlx::migrate!("./migrations")
        .run(&connection_pool)
        .await
        .expect("Failed to migrate database");

    connection_pool
}
