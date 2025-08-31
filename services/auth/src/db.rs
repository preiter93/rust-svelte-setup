use crate::{
    error::DBError,
    utils::{OAuthAccount, Session},
};
use chrono::{DateTime, Utc};
use deadpool_postgres::Pool;
use shared::session::SESSION_TOKEN_EXPIRY_DURATION;
use tonic::async_trait;

#[async_trait]
pub trait DBClient: Send + Sync + 'static {
    async fn insert_session(&self, session: Session) -> Result<(), DBError>;

    async fn get_session(&self, id: &str) -> Result<Session, DBError>;

    async fn delete_session(&self, id: &str) -> Result<(), DBError>;

    async fn update_session(&self, id: &str, expires_at: &DateTime<Utc>) -> Result<(), DBError>;

    async fn upsert_oauth_account(
        &self,
        oauth_account: &OAuthAccount,
    ) -> Result<OAuthAccount, DBError>;

    async fn update_oauth_account(&self, id: &str, user_id: &str) -> Result<OAuthAccount, DBError>;
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
    async fn insert_session(&self, session: Session) -> Result<(), DBError> {
        let client = self.pool.get().await?;
        let expires_at = session
            .created_at
            .checked_add_signed(SESSION_TOKEN_EXPIRY_DURATION);

        client
            .execute(
                "INSERT INTO sessions (id, secret_hash, user_id, created_at, expires_at) VALUES ($1, $2, $3, $4, $5)",
                &[&session.id, &session.secret_hash, &session.user_id, &session.created_at, &expires_at],
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

        let session = Session::try_from(&row)?;

        Ok(session)
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

    /// Inserts or updates an oauth account. Returns the current user_id after upsert.
    ///
    /// # Errors
    /// - database connection cannot be established
    /// - executing database statement fails
    async fn upsert_oauth_account(&self, token: &OAuthAccount) -> Result<OAuthAccount, DBError> {
        let client = self.pool.get().await?;

        let row = client
            .query_one(
                "INSERT INTO oauth_accounts (id, provider, provider_user_id, provider_user_name, provider_user_email, access_token, access_token_expires_at, refresh_token, user_id)
                 VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9)
                 ON CONFLICT (provider_user_id) DO UPDATE SET
                    access_token = EXCLUDED.access_token,
                    access_token_expires_at = EXCLUDED.access_token_expires_at,
                    refresh_token = EXCLUDED.refresh_token,
                    updated_at = NOW()
                 RETURNING id, provider, provider_user_id, provider_user_name, provider_user_email, access_token, access_token_expires_at, refresh_token, user_id",
                &[
                    &token.id,
                    &token.provider,
                    &token.provider_user_id,
                    &token.provider_user_name,
                    &token.provider_user_email,
                    &token.access_token,
                    &token.access_token_expires_at,
                    &token.refresh_token,
                    &token.user_id,
                ],
            )
            .await?;

        let oauth_account = OAuthAccount::try_from(&row)?;

        Ok(oauth_account)
    }

    /// Updates the user id of an oauth account.
    ///
    /// # Errors
    /// - database connection cannot be established
    /// - not found if the row does not exist
    /// - executing database statement fails
    async fn update_oauth_account(&self, id: &str, user_id: &str) -> Result<OAuthAccount, DBError> {
        let client = self.pool.get().await?;

        let row = client
            .query_opt(
                "UPDATE oauth_accounts 
                 SET user_id = $2, updated_at = NOW()
                 WHERE id = $1 
                 RETURNING id, provider, provider_user_id, provider_user_name, provider_user_email, access_token, access_token_expires_at, refresh_token, user_id",
                &[&id, &user_id],
            )
            .await?;
        let Some(row) = row else {
            return Err(DBError::NotFound);
        };

        let oauth_account = OAuthAccount::try_from(&row)?;

        Ok(oauth_account)
    }
}

#[cfg(test)]
pub(crate) mod test {
    use super::*;
    use crate::utils::tests::{fixture_db_oauth_account, fixture_session};
    use crate::{SERVICE_NAME, error::DBError};
    use chrono::TimeZone;
    use shared::test_utils::get_test_db;
    use std::sync::Arc;
    use tokio::sync::Mutex;
    use tonic::async_trait;

    pub(crate) struct MockDBClient {
        pub(crate) insert_session: Mutex<Option<Result<(), DBError>>>,
        pub(crate) get_session: Mutex<Option<Result<Session, DBError>>>,
        pub(crate) delete_session: Mutex<Option<Result<(), DBError>>>,
        pub(crate) update_session: Mutex<Option<Result<(), DBError>>>,
        pub(crate) update_session_count: Arc<Mutex<usize>>,
        pub(crate) delete_session_count: Arc<Mutex<usize>>,
        pub(crate) upsert_oauth_account: Mutex<Option<Result<OAuthAccount, DBError>>>,
        pub(crate) update_oauth_account: Mutex<Option<Result<OAuthAccount, DBError>>>,
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
                upsert_oauth_account: Mutex::new(None),
                update_oauth_account: Mutex::new(None),
            }
        }
    }

    #[async_trait]
    impl DBClient for MockDBClient {
        async fn insert_session(&self, _: Session) -> Result<(), DBError> {
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

        async fn upsert_oauth_account(&self, _: &OAuthAccount) -> Result<OAuthAccount, DBError> {
            self.upsert_oauth_account.lock().await.take().unwrap()
        }

        async fn update_oauth_account(&self, _: &str, _: &str) -> Result<OAuthAccount, DBError> {
            self.update_oauth_account.lock().await.take().unwrap()
        }
    }

    async fn run_db_session_test<F, Fut>(given_sessions: Vec<Session>, test_fn: F)
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
                .insert_session(session)
                .await
                .expect("failed to insert session");
        }

        test_fn(db_client).await;
    }

    async fn run_db_oauth_accounts_test<F, Fut>(given_tokens: Vec<OAuthAccount>, test_fn: F)
    where
        F: FnOnce(PostgresDBClient) -> Fut,
        Fut: std::future::Future<Output = ()>,
    {
        let migrations = std::fs::canonicalize("./migrations").unwrap();
        let db = get_test_db(SERVICE_NAME, migrations)
            .await
            .expect("failed to get test db client");
        let db_client = PostgresDBClient::new(db.pool.clone());

        for token in given_tokens {
            db_client
                .upsert_oauth_account(&token)
                .await
                .expect("failed to insert token");
        }

        test_fn(db_client).await;
    }

    #[tokio::test]
    async fn test_get_session() {
        let session_id = "session-id-get";
        let session = fixture_session(|s| s.id = session_id.to_string());

        run_db_session_test(vec![session.clone()], |db_client| async move {
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

        run_db_session_test(vec![session.clone()], |db_client| async move {
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

        run_db_session_test(vec![session.clone()], |db_client| async move {
            db_client
                .delete_session(session_id)
                .await
                .expect("failed to delete session");

            let got_result = db_client.get_session(session_id).await;

            assert!(matches!(got_result, Err(DBError::NotFound)));
        })
        .await;
    }

    #[tokio::test]
    async fn test_get_oauth_account() {
        let oauth_id = "oauth-id-get";
        let provider_user_id = "provider-user-id-get";

        run_db_oauth_accounts_test(vec![], |db_client| async move {
            let token = fixture_db_oauth_account(|v| {
                v.id = oauth_id.to_string();
                v.provider_user_id = provider_user_id.to_string();
            });
            let got_token = db_client
                .upsert_oauth_account(&token)
                .await
                .expect("failed to insert token");

            assert_eq!(got_token, token);

            let new_token = fixture_db_oauth_account(|v| {
                v.id = oauth_id.to_string();
                v.provider_user_id = provider_user_id.to_string();
                v.access_token = Some(String::from("access-token"));
            });
            let got_token = db_client
                .upsert_oauth_account(&new_token)
                .await
                .expect("failed to upsert token");

            assert_eq!(got_token, new_token);
        })
        .await;
    }

    #[tokio::test]
    async fn test_update_oauth_account() {
        let oauth_id = "oauth-id-update";
        let provider_user_id = "provider-user-id-update";

        let mut token = fixture_db_oauth_account(|v| {
            v.id = oauth_id.to_string();
            v.provider_user_id = provider_user_id.to_string();
        });

        run_db_oauth_accounts_test(vec![token.clone()], |db_client| async move {
            let user_id = "new-user-id";
            token.user_id = Some(user_id.to_string());

            let got_token = db_client
                .update_oauth_account(&oauth_id, &user_id)
                .await
                .expect("failed to update token");

            assert_eq!(got_token, token);
        })
        .await;
    }
}
