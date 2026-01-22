//! PostgreSQL database provisioning

use crate::config::Config;
use crate::error::{Error, Result};

/// Create a PostgreSQL database
pub async fn create_postgres_database(config: &Config, db_name: &str) -> Result<()> {
    let postgres_config = config
        .get_service("postgres")
        .ok_or_else(|| Error::ServiceNotFound("postgres".to_string()))?;

    // Get connection info
    let host = postgres_config
        .connection
        .as_ref()
        .map(|c| c.host.as_str())
        .unwrap_or("localhost");

    let port = postgres_config
        .connection
        .as_ref()
        .map(|c| c.port)
        .unwrap_or(5432);

    let user = postgres_config
        .connection
        .as_ref()
        .map(|c| c.user.as_str())
        .or_else(|| postgres_config.env.get("POSTGRES_USER").map(|s| s.as_str()))
        .unwrap_or("postgres");

    let password = postgres_config
        .connection
        .as_ref()
        .map(|c| c.password.as_str())
        .or_else(|| postgres_config.env.get("POSTGRES_PASSWORD").map(|s| s.as_str()))
        .unwrap_or("postgres");

    let conn_string = format!(
        "host={} port={} user={} password={} dbname=postgres",
        host, port, user, password
    );

    // Connect to postgres
    let (client, connection) = tokio_postgres::connect(&conn_string, tokio_postgres::NoTls)
        .await
        .map_err(|e| Error::Database(e))?;

    // Spawn the connection handler
    tokio::spawn(async move {
        if let Err(e) = connection.await {
            tracing::error!("PostgreSQL connection error: {}", e);
        }
    });

    // Check if database exists
    let exists = client
        .query_one(
            "SELECT EXISTS(SELECT 1 FROM pg_database WHERE datname = $1)",
            &[&db_name],
        )
        .await?;

    let db_exists: bool = exists.get(0);

    if !db_exists {
        // Create database (can't use parameterized query for CREATE DATABASE)
        // Validate db_name to prevent SQL injection
        if !db_name.chars().all(|c| c.is_alphanumeric() || c == '_') {
            return Err(Error::Config(format!(
                "Invalid database name: {}",
                db_name
            )));
        }

        let query = format!("CREATE DATABASE {}", db_name);
        client.execute(&query, &[]).await?;
        tracing::info!("Created database: {}", db_name);
    } else {
        tracing::debug!("Database already exists: {}", db_name);
    }

    Ok(())
}

/// Drop a PostgreSQL database
pub async fn drop_postgres_database(config: &Config, db_name: &str) -> Result<()> {
    let postgres_config = config
        .get_service("postgres")
        .ok_or_else(|| Error::ServiceNotFound("postgres".to_string()))?;

    let host = postgres_config
        .connection
        .as_ref()
        .map(|c| c.host.as_str())
        .unwrap_or("localhost");

    let port = postgres_config
        .connection
        .as_ref()
        .map(|c| c.port)
        .unwrap_or(5432);

    let user = postgres_config
        .connection
        .as_ref()
        .map(|c| c.user.as_str())
        .or_else(|| postgres_config.env.get("POSTGRES_USER").map(|s| s.as_str()))
        .unwrap_or("postgres");

    let password = postgres_config
        .connection
        .as_ref()
        .map(|c| c.password.as_str())
        .or_else(|| postgres_config.env.get("POSTGRES_PASSWORD").map(|s| s.as_str()))
        .unwrap_or("postgres");

    let conn_string = format!(
        "host={} port={} user={} password={} dbname=postgres",
        host, port, user, password
    );

    let (client, connection) = tokio_postgres::connect(&conn_string, tokio_postgres::NoTls)
        .await
        .map_err(|e| Error::Database(e))?;

    tokio::spawn(async move {
        if let Err(e) = connection.await {
            tracing::error!("PostgreSQL connection error: {}", e);
        }
    });

    // Validate db_name
    if !db_name.chars().all(|c| c.is_alphanumeric() || c == '_') {
        return Err(Error::Config(format!(
            "Invalid database name: {}",
            db_name
        )));
    }

    // Terminate connections to the database
    let terminate_query = format!(
        "SELECT pg_terminate_backend(pid) FROM pg_stat_activity WHERE datname = '{}'",
        db_name
    );
    let _ = client.execute(&terminate_query, &[]).await;

    // Drop database
    let query = format!("DROP DATABASE IF EXISTS {}", db_name);
    client.execute(&query, &[]).await?;
    tracing::info!("Dropped database: {}", db_name);

    Ok(())
}

/// List all scratchpad databases
pub async fn list_databases(config: &Config) -> Result<Vec<String>> {
    let postgres_config = config
        .get_service("postgres")
        .ok_or_else(|| Error::ServiceNotFound("postgres".to_string()))?;

    let host = postgres_config
        .connection
        .as_ref()
        .map(|c| c.host.as_str())
        .unwrap_or("localhost");

    let port = postgres_config
        .connection
        .as_ref()
        .map(|c| c.port)
        .unwrap_or(5432);

    let user = postgres_config
        .connection
        .as_ref()
        .map(|c| c.user.as_str())
        .or_else(|| postgres_config.env.get("POSTGRES_USER").map(|s| s.as_str()))
        .unwrap_or("postgres");

    let password = postgres_config
        .connection
        .as_ref()
        .map(|c| c.password.as_str())
        .or_else(|| postgres_config.env.get("POSTGRES_PASSWORD").map(|s| s.as_str()))
        .unwrap_or("postgres");

    let conn_string = format!(
        "host={} port={} user={} password={} dbname=postgres",
        host, port, user, password
    );

    let (client, connection) = tokio_postgres::connect(&conn_string, tokio_postgres::NoTls)
        .await
        .map_err(|e| Error::Database(e))?;

    tokio::spawn(async move {
        if let Err(e) = connection.await {
            tracing::error!("PostgreSQL connection error: {}", e);
        }
    });

    let rows = client
        .query(
            "SELECT datname FROM pg_database WHERE datname LIKE 'scratch_%'",
            &[],
        )
        .await?;

    let databases: Vec<String> = rows.iter().map(|row| row.get(0)).collect();
    Ok(databases)
}
