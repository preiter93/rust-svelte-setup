use crate::utils::UuidGenerator;

use crate::{
    db::DBClient,
    error::{DBError, Error},
    proto::{
        CreateUserReq, CreateUserResp, GetUserReq, GetUserResp, User,
        api_service_server::ApiService,
    },
};
use shared::helper::validate_user_id;
use tonic::{Request, Response, Status};
use tracing::instrument;

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
    #[instrument(skip_all, fields(user_id), err)]
    async fn create_user(
        &self,
        req: Request<CreateUserReq>,
    ) -> Result<Response<CreateUserResp>, Status> {
        let req = req.into_inner();
        let id = self.uuid.generate();

        tracing::Span::current().record("user_id", &id.to_string());

        let name = req.name;
        if name.is_empty() {
            return Err(Error::MissingUserName.into());
        }

        let email = req.email;
        if email.is_empty() {
            return Err(Error::MissingUserEmail.into());
        }

        self.db
            .insert_user(id, &name, &email)
            .await
            .map_err(Error::InsertUser)?;

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
    #[instrument(skip_all, fields(user_id), err)]
    async fn get_user(&self, req: Request<GetUserReq>) -> Result<Response<GetUserResp>, Status> {
        let req = req.into_inner();
        let user_id = validate_user_id(&req.id)?;

        let user = self.db.get_user(user_id).await.map_err(|e| match e {
            DBError::NotFound => Error::UserNotFound(user_id.to_string()),
            _ => Error::GetUser(e),
        })?;

        Ok(Response::new(GetUserResp { user: Some(user) }))
    }
}

#[cfg(test)]
mod tests {
    use tokio::sync::Mutex;
    use tonic::{Code, Request};

    use crate::{
        db::test::MockDBClient,
        error::DBError,
        handler::Handler,
        proto::{
            CreateUserReq, CreateUserResp, GetUserReq, GetUserResp, User,
            api_service_server::ApiService as _,
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
}
