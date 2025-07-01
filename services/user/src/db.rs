use deadpool_postgres::Pool;
use thiserror::Error;
use tokio_postgres::Row;
use uuid::Uuid;

use crate::proto::User;

#[derive(Clone)]
pub struct DBCLient {
    pub pool: Pool,
}

impl DBCLient {
    #[must_use]
    pub fn new(pool: Pool) -> Self {
        Self { pool }
    }

    /// # Errors
    /// - if the database connection cannot be established
    pub async fn insert_user(&self, id: Uuid) -> Result<(), DBError> {
        let client = self.pool.get().await?;

        client
            .execute("INSERT INTO users (id) VALUES ($1)", &[&id])
            .await?;

        Ok(())
    }

    /// # Errors
    /// - if the database connection cannot be established
    /// - If the user is not found
    pub async fn get_user(&self, id: Uuid) -> Result<User, DBError> {
        let client = self.pool.get().await?;

        let stmt = client.prepare("SELECT id FROM users WHERE id = $1").await?;
        let row = client.query_opt(&stmt, &[&id]).await?;
        let Some(row) = row else {
            return Err(DBError::NotFound);
        };

        let user: User = User::try_from(row)?;

        Ok(user)
    }

    /// # Errors
    /// - if the database connection cannot be established
    pub async fn list_users(&self) -> Result<Vec<User>, DBError> {
        let client = self.pool.get().await?;

        let stmt = client.prepare("SELECT id FROM users").await?;
        let rows = client.query(&stmt, &[]).await?;

        let users = rows
            .into_iter()
            .flat_map(|row| User::try_from(row))
            .collect::<Vec<_>>();

        Ok(users)
    }
}

impl TryFrom<Row> for User {
    type Error = DBError;

    fn try_from(value: Row) -> Result<Self, DBError> {
        let id: Uuid = value.try_get("id")?;

        Ok(User { id: id.to_string() })
    }
}

#[derive(Debug, Error)]
pub enum DBError {
    #[error("Database connection failed: {0}")]
    Connection(#[from] tokio_postgres::Error),

    #[error("Connection pool error: {0}")]
    Pool(#[from] deadpool_postgres::PoolError),

    #[error("Entity not found")]
    NotFound,

    #[error("Conversion error: {0}")]
    Conversion(String),
}
