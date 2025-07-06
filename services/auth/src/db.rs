use chrono::{DateTime, Utc};
use deadpool_postgres::Pool;
use thiserror::Error;

use crate::utils::Session;

#[derive(Clone)]
pub struct DBCLient {
    pub pool: Pool,
}

impl DBCLient {
    /// Creates a new `DBClient`.
    #[must_use]
    pub fn new(pool: Pool) -> Self {
        Self { pool }
    }

    /// Inserts a session into the database.
    ///
    /// # Errors
    /// - database connection cannot be established
    /// - inserting row into the database fails
    pub async fn insert_session(
        &self,
        id: &str,
        secret_hash: &[u8],
        created_at: DateTime<Utc>,
    ) -> Result<(), DBError> {
        let client = self.pool.get().await?;

        client
            .execute(
                "INSERT INTO sessions (id, secret_hash, created_at) VALUES ($1, $2, $3)",
                &[&id, &secret_hash, &created_at],
            )
            .await?;

        Ok(())
    }

    /// Returns a session from the database.
    ///
    /// # Errors
    /// - database connection cannot be established
    /// - fetching row from the database fails
    pub async fn get_session(&self, id: &str) -> Result<Session, DBError> {
        let client = self.pool.get().await?;

        let stmt = client
            .prepare("SELECT id, secret_hash, created_at FROM sessions WHERE id = $1")
            .await?;
        let row = client.query_opt(&stmt, &[&id]).await?;
        let Some(row) = row else {
            return Err(DBError::NotFound);
        };

        let id: String = row.try_get("id")?;
        let secret_hash: Vec<u8> = row.try_get("secret_hash")?;
        let created_at: DateTime<Utc> = row.try_get("created_at")?;

        Ok(Session {
            id,
            secret_hash,
            created_at,
        })
    }

    /// Deletes a session into the database.
    ///
    /// # Errors
    /// - database connection cannot be established
    /// - deleting row from the database fails
    pub async fn delete_session(&self, id: &str) -> Result<(), DBError> {
        let client = self.pool.get().await?;

        client
            .execute("DELETE FROM sessions WHERE id = $1", &[&id])
            .await?;

        Ok(())
    }
}

#[derive(Debug, Error)]
pub enum DBError {
    #[error("database connection failed: {0}")]
    Connection(#[from] tokio_postgres::Error),

    #[error("connection pool error: {0}")]
    Pool(#[from] deadpool_postgres::PoolError),

    #[error("entity not found")]
    NotFound,

    #[error("conversion error: {0}")]
    Conversion(String),
}

impl From<DBError> for tonic::Status {
    fn from(value: DBError) -> Self {
        Self::internal(value.to_string())
    }
}
