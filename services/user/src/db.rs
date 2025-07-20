use crate::proto::get_user_req;
use deadpool_postgres::Pool;
use thiserror::Error;
use tokio_postgres::Row;
use uuid::Uuid;

use crate::proto::User;

#[derive(Clone, Debug)]
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
    /// - if the database query fails
    pub async fn insert_user(&self, id: Uuid, google_id: &str) -> Result<(), DBError> {
        let client = self.pool.get().await?;

        client
            .execute(
                "INSERT INTO users (id, google_id) VALUES ($1, $2)",
                &[&id, &google_id],
            )
            .await?;

        Ok(())
    }

    /// # Errors
    /// - if the database connection cannot be established
    /// - if the database query fails
    /// - If the user is not found
    pub async fn get_user(&self, identifier: get_user_req::Identifier) -> Result<User, DBError> {
        let client = self.pool.get().await?;

        let (stmt, param) = match identifier {
            get_user_req::Identifier::Id(id) => (
                client.prepare("SELECT id FROM users WHERE id = $1").await?,
                id,
            ),
            get_user_req::Identifier::GoogleId(google_id) => (
                client
                    .prepare("SELECT id FROM users WHERE google_id = $1")
                    .await?,
                google_id,
            ),
        };
        let row = client.query_opt(&stmt, &[&param]).await?;
        let Some(row) = row else {
            return Err(DBError::NotFound);
        };

        let user: User = User::try_from(row)?;

        Ok(user)
    }

    /// # Errors
    /// - if the database connection cannot be established
    /// - if the database query fails
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
    #[error("Database error: {0}")]
    Error(#[from] tokio_postgres::Error),

    #[error("Connection pool error: {0}")]
    Pool(#[from] deadpool_postgres::PoolError),

    #[error("Entity not found")]
    NotFound,

    #[error("Conversion error: {0}")]
    Conversion(String),
}
