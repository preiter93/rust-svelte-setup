use crate::{error::DBError, utils::Session};
use chrono::{DateTime, Utc};
use deadpool_postgres::Pool;

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
        user_id: &str,
        created_at: DateTime<Utc>,
    ) -> Result<(), DBError> {
        let client = self.pool.get().await?;

        client
            .execute(
                "INSERT INTO sessions (id, secret_hash, user_id, created_at) VALUES ($1, $2, $3, $4)",
                &[&id, &secret_hash, &user_id, &created_at],
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
            .prepare("SELECT id, secret_hash, created_at, user_id FROM sessions WHERE id = $1")
            .await?;
        let row = client.query_opt(&stmt, &[&id]).await?;
        let Some(row) = row else {
            return Err(DBError::NotFound);
        };

        let id: String = row.try_get("id")?;
        let secret_hash: Vec<u8> = row.try_get("secret_hash")?;
        let created_at: DateTime<Utc> = row.try_get("created_at")?;
        let user_id: String = row.try_get("user_id")?;

        Ok(Session {
            id,
            secret_hash,
            created_at,
            user_id,
        })
    }

    /// Deletes a session from the database.
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
