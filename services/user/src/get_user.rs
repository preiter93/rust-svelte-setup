use crate::{
    db::DBClient,
    error::{DBError, Error},
    proto::{GetUserReq, GetUserResp},
    server::Server,
    utils::UuidGenerator,
};
use setup::validate_user_id;
use tonic::{Request, Response, Status};

impl<D, U> Server<D, U>
where
    D: DBClient,
    U: UuidGenerator,
{
    /// Gets a user by identifier.
    ///
    /// # Errors
    /// - internal error if the user cannot be inserted into the db
    pub async fn get_user(
        &self,
        req: Request<GetUserReq>,
    ) -> Result<Response<GetUserResp>, Status> {
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
        proto::{GetUserReq, GetUserResp, User},
        server::Server,
        utils::test::{MockUuidGenerator, assert_response, fixture_user, fixture_uuid},
    };

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
        let service = Server {
            db,
            uuid: MockUuidGenerator::default(),
        };

        // when
        let got = service.get_user(Request::new(GetUserReq { id })).await;

        // then
        assert_response(got, want);
    }
}
