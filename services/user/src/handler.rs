use crate::utils::UuidGenerator;

use crate::{
    db::DBClient,
    error::{DBError, Error},
    proto::{
        CreateUserReq, CreateUserResp, GetUserReq, GetUserResp, User,
        api_service_server::ApiService,
    },
};
use setup::validate_user_id;
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

        tracing::Span::current().record("user_id", id.to_string());

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
    use rstest::rstest;
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

    #[rstest]
    #[case::happy(
        fixture_create_user_req(|_| {}),
        Ok(()),
        Ok(CreateUserResp { user: Some(fixture_user(|_| {})) })
    )]
    #[case::missing_name(
        fixture_create_user_req(|r| r.name.clear()),
        Ok(()),
        Err(Code::InvalidArgument)
    )]
    #[case::missing_email(
        fixture_create_user_req(|r| r.email.clear()),
        Ok(()),
        Err(Code::InvalidArgument)
    )]
    #[case::internal_error(
        fixture_create_user_req(|_| {}),
        Err(DBError::Unknown),
        Err(Code::Internal)
    )]
    #[tokio::test]
    async fn test_create_user(
        #[case] req: CreateUserReq,
        #[case] insert_res: Result<(), DBError>,
        #[case] want: Result<CreateUserResp, Code>,
    ) {
        let db = MockDBClient {
            insert_user: Mutex::new(Some(insert_res)),
            ..Default::default()
        };

        let service = Handler {
            db,
            uuid: MockUuidGenerator::default(),
        };

        let got = service.create_user(Request::new(req)).await;
        assert_response(got, want);
    }

    #[rstest]
    #[case::happy_path(
        fixture_uuid().to_string(),
        Ok(fixture_user(|_| {})),
        Ok(GetUserResp { user: Some(fixture_user(|_| {})) })
    )]
    #[case::missing_id(
        "".to_string(),
        Ok(fixture_user(|_| {})),
        Err(Code::InvalidArgument)
    )]
    #[case::not_a_uuid(
        "not-uuid".to_string(),
        Ok(fixture_user(|_| {})),
        Err(Code::InvalidArgument)
    )]
    #[case::not_found(
        fixture_uuid().to_string(),
        Err(DBError::NotFound),
        Err(Code::NotFound)
    )]
    #[case::internal_error(
        fixture_uuid().to_string(),
        Err(DBError::Unknown),
        Err(Code::Internal)
    )]
    #[tokio::test]
    async fn test_get_user(
        #[case] id: String,
        #[case] db_result: Result<User, DBError>,
        #[case] want: Result<GetUserResp, Code>,
    ) {
        // given
        let db = MockDBClient {
            get_user: Mutex::new(Some(db_result)),
            ..Default::default()
        };
        let service = Handler {
            db,
            uuid: MockUuidGenerator::default(),
        };

        // when
        let got = service.get_user(Request::new(GetUserReq { id })).await;

        // then
        assert_response(got, want);
    }
}
