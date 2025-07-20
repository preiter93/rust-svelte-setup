use crate::{
    db::{DBCLient, DBError},
    proto::{
        CreateUserReq, CreateUserResp, GetUserReq, GetUserResp, ListUsersReq, ListUsersResp,
        api_service_server::ApiService, get_user_req,
    },
};
use thiserror::Error;
use tonic::{Code, Request, Response, Status};
use tracing::instrument;
use uuid::Uuid;

#[derive(Clone, Debug)]
pub(crate) struct Handler {
    pub db: DBCLient,
}

#[tonic::async_trait]
impl ApiService for Handler {
    /// Creates a new user.
    ///
    /// # Errors
    /// - internal error if the user cannot be inserted into the db
    #[instrument(skip_all)]
    async fn create_user(
        &self,
        req: Request<CreateUserReq>,
    ) -> Result<Response<CreateUserResp>, Status> {
        let req = req.into_inner();
        let user_id = Uuid::new_v4();

        self.db
            .insert_user(user_id, &req.google_id)
            .await
            .map_err(CreateUserErr::Database)?;

        let response = CreateUserResp {
            id: user_id.to_string(),
        };

        Ok(Response::new(response))
    }

    /// Gets a user by id.
    ///
    /// # Errors
    /// - internal error if the user cannot be inserted into the db
    #[instrument(field(req = req.into_inner()))]
    async fn get_user(&self, req: Request<GetUserReq>) -> Result<Response<GetUserResp>, Status> {
        let req = req.into_inner();
        let Some(identifier) = req.identifier else {
            return Err(GetUserErr::MissingIdentifier.into());
        };
        match identifier {
            get_user_req::Identifier::Id(ref id) if id.is_empty() => {
                return Err(GetUserErr::EmptyID.into());
            }
            get_user_req::Identifier::GoogleId(ref google_id) if google_id.is_empty() => {
                return Err(GetUserErr::EmptyGoogleID.into());
            }
            _ => {}
        }

        let user = self.db.get_user(identifier).await.map_err(|e| match e {
            DBError::NotFound => GetUserErr::NotFound,
            _ => GetUserErr::Database(e),
        })?;

        let response = GetUserResp { user: Some(user) };
        Ok(Response::new(response))
    }

    /// Lists all users.
    ///
    /// # Errors
    /// - internal error if the users cannot be retrieved from the db
    #[instrument(skip_all)]
    async fn list_users(
        &self,
        _: Request<ListUsersReq>,
    ) -> Result<Response<ListUsersResp>, Status> {
        let users = self.db.list_users().await.map_err(ListUsersErr::Database)?;

        let response = ListUsersResp { users };
        Ok(Response::new(response))
    }
}

#[derive(Debug, Error)]
#[non_exhaustive]
pub enum CreateUserErr {
    #[error("database error: {0}")]
    Database(#[from] DBError),
}

impl From<CreateUserErr> for Status {
    fn from(err: CreateUserErr) -> Self {
        let code = match err {
            CreateUserErr::Database(_) => Code::Internal,
        };
        Status::new(code, err.to_string())
    }
}

#[derive(Debug, Error)]
#[non_exhaustive]
pub enum GetUserErr {
    #[error("missing identifier")]
    MissingIdentifier,

    #[error("empty id")]
    EmptyID,

    #[error("empty google-id")]
    EmptyGoogleID,

    #[error("user not found")]
    NotFound,

    #[error("database error: {0}")]
    Database(#[from] DBError),
}

impl From<GetUserErr> for Status {
    fn from(err: GetUserErr) -> Self {
        let code = match err {
            GetUserErr::MissingIdentifier | GetUserErr::EmptyID | GetUserErr::EmptyGoogleID => {
                Code::InvalidArgument
            }
            GetUserErr::NotFound => Code::NotFound,
            GetUserErr::Database(_) => Code::Internal,
        };
        Status::new(code, err.to_string())
    }
}

#[derive(Debug, Error)]
#[non_exhaustive]
pub enum ListUsersErr {
    #[error("database error: {0}")]
    Database(#[from] DBError),
}

impl From<ListUsersErr> for Status {
    fn from(err: ListUsersErr) -> Self {
        let code = match err {
            ListUsersErr::Database(_) => Code::Internal,
        };
        Status::new(code, err.to_string())
    }
}
