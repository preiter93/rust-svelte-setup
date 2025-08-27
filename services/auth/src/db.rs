use crate::{error::DBError, utils::Session};
use chrono::{DateTime, Utc};
use deadpool_postgres::Pool;
use shared::session::SESSION_TOKEN_EXPIRY_DURATION;
use tonic::async_trait;

#[async_trait]
pub trait DBClient: Send + Sync + 'static {
    async fn insert_session(
        &self,
        id: &str,
        secret_hash: &[u8],
        user_id: &str,
        created_at: DateTime<Utc>,
    ) -> Result<(), DBError>;

    async fn get_session(&self, id: &str) -> Result<Session, DBError>;

    async fn delete_session(&self, id: &str) -> Result<(), DBError>;

    async fn update_session(&self, id: &str, expires_at: &DateTime<Utc>) -> Result<(), DBError>;
}

#[derive(Clone)]
pub struct PostgresDBClient {
    pub pool: Pool,
}

impl PostgresDBClient {
    /// Creates a new `DBClient`.
    #[must_use]
    pub fn new(pool: Pool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl DBClient for PostgresDBClient {
    /// Inserts a session into the database.
    ///
    /// # Errors
    /// - database connection cannot be established
    /// - executing database statement fails
    async fn insert_session(
        &self,
        id: &str,
        secret_hash: &[u8],
        user_id: &str,
        created_at: DateTime<Utc>,
    ) -> Result<(), DBError> {
        let client = self.pool.get().await?;
        let expires_at = created_at.checked_add_signed(SESSION_TOKEN_EXPIRY_DURATION);

        client
            .execute(
                "INSERT INTO sessions (id, secret_hash, user_id, created_at, expires_at) VALUES ($1, $2, $3, $4, $5)",
                &[&id, &secret_hash, &user_id, &created_at, &expires_at],
            )
            .await?;

        Ok(())
    }

    /// Returns a session from the database.
    ///
    /// # Errors
    /// - not found
    /// - database connection cannot be established
    /// - executing database statement fails
    async fn get_session(&self, id: &str) -> Result<Session, DBError> {
        let client = self.pool.get().await?;

        let stmt = client
            .prepare("SELECT id, secret_hash, created_at, expires_at, user_id FROM sessions WHERE id = $1")
            .await?;
        let row = client.query_opt(&stmt, &[&id]).await?;
        let Some(row) = row else {
            return Err(DBError::NotFound);
        };

        let id: String = row.try_get("id")?;
        let secret_hash: Vec<u8> = row.try_get("secret_hash")?;
        let created_at: DateTime<Utc> = row.try_get("created_at")?;
        let expires_at: DateTime<Utc> = row.try_get("expires_at")?;
        let user_id: String = row.try_get("user_id")?;

        Ok(Session {
            id,
            secret_hash,
            created_at,
            expires_at,
            user_id,
        })
    }

    /// Deletes a session from the database.
    ///
    /// # Errors
    /// - database connection cannot be established
    /// - executing database statement fails
    async fn delete_session(&self, id: &str) -> Result<(), DBError> {
        let client = self.pool.get().await?;

        client
            .execute("DELETE FROM sessions WHERE id = $1", &[&id])
            .await?;

        Ok(())
    }

    /// Updates a session in the database.
    ///
    /// # Errors
    /// - database connection cannot be established
    /// - executing database statement fails
    async fn update_session(&self, id: &str, expires_at: &DateTime<Utc>) -> Result<(), DBError> {
        let client = self.pool.get().await?;

        client
            .execute(
                "UPDATE sessions SET expires_at = $1 WHERE id = $2",
                &[&expires_at, &id],
            )
            .await?;

        Ok(())
    }
}

#[cfg(test)]
pub(crate) mod test {
    use crate::SERVICE_NAME;
    use crate::utils::tests::fixture_session;
    use chrono::TimeZone;
    use shared::test_utils::get_test_db;
    use std::sync::Arc;

    use tokio::sync::Mutex;
    use tonic::async_trait;

    use super::*;

    use crate::error::DBError;

    pub(crate) struct MockDBClient {
        pub(crate) insert_session: Mutex<Option<Result<(), DBError>>>,
        pub(crate) get_session: Mutex<Option<Result<Session, DBError>>>,
        pub(crate) delete_session: Mutex<Option<Result<(), DBError>>>,
        pub(crate) update_session: Mutex<Option<Result<(), DBError>>>,
        pub(crate) update_session_count: Arc<Mutex<usize>>,
        pub(crate) delete_session_count: Arc<Mutex<usize>>,
    }

    impl Default for MockDBClient {
        fn default() -> Self {
            Self {
                insert_session: Mutex::new(None),
                get_session: Mutex::new(None),
                delete_session: Mutex::new(None),
                update_session: Mutex::new(None),
                update_session_count: Arc::new(Mutex::new(0)),
                delete_session_count: Arc::new(Mutex::new(0)),
            }
        }
    }

    #[async_trait]
    impl DBClient for MockDBClient {
        async fn insert_session(
            &self,
            _: &str,
            _: &[u8],
            _: &str,
            _: DateTime<Utc>,
        ) -> Result<(), DBError> {
            self.insert_session.lock().await.take().unwrap()
        }

        async fn get_session(&self, _: &str) -> Result<Session, DBError> {
            self.get_session.lock().await.take().unwrap()
        }

        async fn delete_session(&self, _: &str) -> Result<(), DBError> {
            let mut count = self.delete_session_count.lock().await;
            *count += 1;
            self.delete_session.lock().await.take().unwrap()
        }

        async fn update_session(&self, _: &str, _: &DateTime<Utc>) -> Result<(), DBError> {
            let mut count = self.update_session_count.lock().await;
            *count += 1;
            self.update_session.lock().await.take().unwrap()
        }
    }

    async fn run_db_test<F, Fut>(given_sessions: Vec<Session>, test_fn: F)
    where
        F: FnOnce(PostgresDBClient) -> Fut,
        Fut: std::future::Future<Output = ()>,
    {
        let migrations = std::fs::canonicalize("./migrations").unwrap();
        let db = get_test_db(SERVICE_NAME, migrations)
            .await
            .expect("failed to get test db client");
        let db_client = PostgresDBClient::new(db.pool.clone());

        for session in given_sessions {
            db_client
                .insert_session(
                    &session.id,
                    &session.secret_hash,
                    &session.user_id,
                    session.created_at,
                )
                .await
                .expect("failed to insert session");
        }

        test_fn(db_client).await;
    }

    #[tokio::test]
    async fn test_get_session() {
        let session_id = "session-id-get";
        let session = fixture_session(|s| s.id = session_id.to_string());

        run_db_test(vec![session.clone()], |db_client| async move {
            let got_session = db_client
                .get_session(session_id)
                .await
                .expect("failed to get session");

            assert_eq!(got_session, session);
        })
        .await;
    }

    #[tokio::test]
    async fn test_update_session() {
        let session_id = "session-id-update";
        let mut session = fixture_session(|s| s.id = session_id.to_string());

        run_db_test(vec![session.clone()], |db_client| async move {
            session.expires_at = chrono::Utc.with_ymd_and_hms(2020, 1, 9, 0, 0, 0).unwrap();
            db_client
                .update_session(session_id, &session.expires_at)
                .await
                .expect("failed to update session");

            let got_session = db_client
                .get_session(session_id)
                .await
                .expect("failed to get session");

            assert_eq!(got_session, session);
        })
        .await;
    }

    #[tokio::test]
    async fn test_delete_session() {
        let session_id = "session-id-delete";
        let session = fixture_session(|s| s.id = session_id.to_string());

        run_db_test(vec![session.clone()], |db_client| async move {
            db_client
                .delete_session(session_id)
                .await
                .expect("failed to delete session");

            let got_result = db_client.get_session(session_id).await;

            assert!(matches!(got_result, Err(DBError::NotFound)));
        })
        .await;
    }
}
