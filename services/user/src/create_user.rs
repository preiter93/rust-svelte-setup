use crate::{
    db::DBClient,
    error::Error,
    handler::Handler,
    proto::{CreateUserReq, CreateUserResp, User},
    utils::UuidGenerator,
};
use tonic::{Request, Response, Status};

impl<D, U> Handler<D, U>
where
    D: DBClient,
    U: UuidGenerator,
{
    /// Creates a new user.
    ///
    /// # Errors
    /// - internal error if the user cannot be inserted into the db
    pub async fn create_user(
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
        proto::{CreateUserReq, CreateUserResp},
        utils::test::{MockUuidGenerator, assert_response, fixture_create_user_req, fixture_user},
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
}
