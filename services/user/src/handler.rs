use crate::{proto::OauthProvider, utils::UuidGenerator};
use std::str::FromStr;

use crate::{
    db::DBClient,
    error::{CreateUserErr, DBError, GetUserErr, GetUserIdFromOauthIdErr},
    proto::{
        CreateUserReq, CreateUserResp, GetUserIdFromOauthIdReq, GetUserIdFromOauthIdResp,
        GetUserReq, GetUserResp, User, api_service_server::ApiService,
    },
};
use tonic::{Request, Response, Status};
use tracing::instrument;
use uuid::Uuid;

#[derive(Clone)]
pub(crate) struct Handler<D, U> {
    pub db: D,
    pub uuid: U,
}

#[tonic::async_trait]
impl<D, U> ApiService for Handler<D, U>
where
    D: DBClient,
    U: UuidGenerator,
{
    /// Creates a new user.
    ///
    /// # Errors
    /// - internal error if the user cannot be inserted into the db
    #[instrument(skip_all, err)]
    async fn create_user(
        &self,
        req: Request<CreateUserReq>,
    ) -> Result<Response<CreateUserResp>, Status> {
        let req = req.into_inner();
        let id = self.uuid.new();

        let name = req.name;
        if name.is_empty() {
            return Err(CreateUserErr::MissingName.into());
        }

        let email = req.email;
        if email.is_empty() {
            return Err(CreateUserErr::MissingEmail.into());
        }

        let google_id = req.google_id;
        let github_id = req.github_id;

        self.db
            .insert_user(id, &name, &email, &google_id, &github_id)
            .await
            .map_err(CreateUserErr::Database)?;

        let response = CreateUserResp {
            user: Some(User {
                id: id.to_string(),
                name,
                email,
            }),
        };

        Ok(Response::new(response))
    }

    /// Gets a user by identifier.
    ///
    /// # Errors
    /// - internal error if the user cannot be inserted into the db
    #[instrument(skip_all, err)]
    async fn get_user(&self, req: Request<GetUserReq>) -> Result<Response<GetUserResp>, Status> {
        let req = req.into_inner();
        if req.id.is_empty() {
            return Err(GetUserErr::MissingUserId.into());
        }
        let id = Uuid::from_str(&req.id).map_err(|_| GetUserErr::NotAUUID)?;

        let user = self.db.get_user(id).await.map_err(|e| match e {
            DBError::NotFound => GetUserErr::NotFound,
            _ => GetUserErr::Database(e),
        })?;

        let response = GetUserResp { user: Some(user) };
        Ok(Response::new(response))
    }

    /// Gets a user id by google id.
    ///
    /// # Errors
    /// - internal error if the user cannot be inserted into the db
    #[instrument(skip_all, err)]
    async fn get_user_id_from_oauth_id(
        &self,
        req: Request<GetUserIdFromOauthIdReq>,
    ) -> Result<Response<GetUserIdFromOauthIdResp>, Status> {
        let req = req.into_inner();

        let oauth_id = req.oauth_id.clone();
        if oauth_id.is_empty() {
            return Err(GetUserIdFromOauthIdErr::MissingOAuthId.into());
        }

        let provider = req.provider();
        if provider == OauthProvider::Unspecified {
            return Err(GetUserIdFromOauthIdErr::UnspecifiedOauthProvider.into());
        }

        let id = self
            .db
            .get_user_id_from_oauth_id(&oauth_id, provider)
            .await
            .map_err(|e| match e {
                DBError::NotFound => GetUserIdFromOauthIdErr::NotFound,
                _ => GetUserIdFromOauthIdErr::Database(e),
            })?;

        let response = GetUserIdFromOauthIdResp { id: id.to_string() };
        Ok(Response::new(response))
    }
}

#[cfg(test)]
mod tests {
    use tokio::sync::Mutex;
    use tonic::{Code, Request};
    use user::proto::OauthProvider;
    use uuid::Uuid;

    use crate::{
        db::test::MockDBClient,
        error::DBError,
        handler::Handler,
        proto::{
            CreateUserReq, CreateUserResp, GetUserIdFromOauthIdReq, GetUserIdFromOauthIdResp,
            GetUserReq, GetUserResp, User, api_service_server::ApiService as _,
        },
        utils::test::{
            MockUuidGenerator, assert_response, fixture_create_user_req, fixture_user, fixture_uuid,
        },
    };

    struct TestCaseCreateUser {
        given_req: CreateUserReq,
        given_db_insert_user: Result<(), DBError>,
        want: Result<CreateUserResp, Code>,
    }

    impl Default for TestCaseCreateUser {
        fn default() -> Self {
            Self {
                given_req: fixture_create_user_req(|_| {}),
                given_db_insert_user: Ok(()),
                want: Ok(CreateUserResp {
                    user: Some(fixture_user(|_| {})),
                }),
            }
        }
    }

    impl TestCaseCreateUser {
        async fn run(self) {
            // given
            let db = MockDBClient {
                insert_user: Mutex::new(Some(self.given_db_insert_user)),
                ..Default::default()
            };
            let uuid = MockUuidGenerator::default();
            let service = Handler { db, uuid };

            // when
            let req = Request::new(self.given_req);
            let got = service.create_user(req).await;

            // then
            assert_response(got, self.want);
        }
    }

    #[tokio::test]
    async fn test_create_user_happy_path() {
        TestCaseCreateUser {
            ..Default::default()
        }
        .run()
        .await;
    }

    #[tokio::test]
    async fn test_create_user_missing_name() {
        TestCaseCreateUser {
            given_req: fixture_create_user_req(|r| r.name = String::new()),
            want: Err(Code::InvalidArgument),
            ..Default::default()
        }
        .run()
        .await;
    }

    #[tokio::test]
    async fn test_create_user_missing_email() {
        TestCaseCreateUser {
            given_req: fixture_create_user_req(|r| r.email = String::new()),
            want: Err(Code::InvalidArgument),
            ..Default::default()
        }
        .run()
        .await;
    }

    #[tokio::test]
    async fn test_create_user_internal_error() {
        TestCaseCreateUser {
            given_db_insert_user: Err(DBError::Unknown),
            want: Err(Code::Internal),
            ..Default::default()
        }
        .run()
        .await;
    }

    struct TestCaseGetUser {
        given_id: String,
        given_db_get_user: Result<User, DBError>,
        want: Result<GetUserResp, Code>,
    }

    impl Default for TestCaseGetUser {
        fn default() -> Self {
            Self {
                given_id: fixture_uuid().to_string(),
                given_db_get_user: Ok(fixture_user(|_| {})),
                want: Ok(GetUserResp {
                    user: Some(fixture_user(|_| {})),
                }),
            }
        }
    }

    impl TestCaseGetUser {
        async fn run(self) {
            // given
            let db = MockDBClient {
                get_user: Mutex::new(Some(self.given_db_get_user)),
                ..Default::default()
            };
            let uuid = MockUuidGenerator::default();
            let service = Handler { db, uuid };

            // when
            let req = Request::new(GetUserReq { id: self.given_id });
            let got = service.get_user(req).await;

            // then
            assert_response(got, self.want);
        }
    }

    #[tokio::test]
    async fn test_get_user_happy_path() {
        TestCaseGetUser {
            ..Default::default()
        }
        .run()
        .await;
    }

    #[tokio::test]
    async fn test_get_user_missing_id() {
        TestCaseGetUser {
            given_id: String::new(),
            want: Err(Code::InvalidArgument),
            ..Default::default()
        }
        .run()
        .await;
    }

    #[tokio::test]
    async fn test_get_user_not_a_uuid() {
        TestCaseGetUser {
            given_id: "not-uuid".to_string(),
            want: Err(Code::InvalidArgument),
            ..Default::default()
        }
        .run()
        .await;
    }

    #[tokio::test]
    async fn test_get_user_not_found() {
        TestCaseGetUser {
            given_db_get_user: Err(DBError::NotFound),
            want: Err(Code::NotFound),
            ..Default::default()
        }
        .run()
        .await;
    }

    #[tokio::test]
    async fn test_get_user_internal_error() {
        TestCaseGetUser {
            given_db_get_user: Err(DBError::Unknown),
            want: Err(Code::Internal),
            ..Default::default()
        }
        .run()
        .await;
    }

    struct TestCaseGetUserIdFromOauthId {
        given_google_id: String,
        given_db_get_user_id_from_google_id: Result<Uuid, DBError>,
        want: Result<GetUserIdFromOauthIdResp, Code>,
    }

    impl Default for TestCaseGetUserIdFromOauthId {
        fn default() -> Self {
            Self {
                given_google_id: fixture_uuid().to_string(),
                given_db_get_user_id_from_google_id: Ok(fixture_uuid()),
                want: Ok(GetUserIdFromOauthIdResp {
                    id: fixture_uuid().to_string(),
                }),
            }
        }
    }

    impl TestCaseGetUserIdFromOauthId {
        async fn run(self) {
            // given
            let db = MockDBClient {
                get_user_id_from_oauth_id: Mutex::new(Some(
                    self.given_db_get_user_id_from_google_id,
                )),
                ..Default::default()
            };
            let uuid = MockUuidGenerator::default();
            let service = Handler { db, uuid };

            // when
            let req = Request::new(GetUserIdFromOauthIdReq {
                oauth_id: self.given_google_id,
                provider: OauthProvider::Google.into(),
            });
            let got = service.get_user_id_from_oauth_id(req).await;

            // then
            assert_response(got, self.want);
        }
    }

    #[tokio::test]
    async fn test_get_user_id_from_google_id_happy_path() {
        TestCaseGetUserIdFromOauthId {
            ..Default::default()
        }
        .run()
        .await;
    }

    #[tokio::test]
    async fn test_get_user_id_from_google_id_missing_id() {
        TestCaseGetUserIdFromOauthId {
            given_google_id: String::new(),
            want: Err(Code::InvalidArgument),
            ..Default::default()
        }
        .run()
        .await;
    }

    #[tokio::test]
    async fn test_get_user_id_from_google_id_not_found() {
        TestCaseGetUserIdFromOauthId {
            given_db_get_user_id_from_google_id: Err(DBError::NotFound),
            want: Err(Code::NotFound),
            ..Default::default()
        }
        .run()
        .await;
    }

    #[tokio::test]
    async fn test_get_user_id_from_google_id_internal_error() {
        TestCaseGetUserIdFromOauthId {
            given_db_get_user_id_from_google_id: Err(DBError::Unknown),
            want: Err(Code::Internal),
            ..Default::default()
        }
        .run()
        .await;
    }
}
