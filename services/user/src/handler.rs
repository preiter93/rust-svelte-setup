use std::str::FromStr;

use crate::{
    db::{DBCLient, DBError},
    proto::{
        CreateUserReq, CreateUserResp, GetUserIdFromGoogleIdReq, GetUserIdFromGoogleIdResp,
        GetUserReq, GetUserResp, User, api_service_server::ApiService,
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
    #[instrument(skip_all, err)]
    async fn create_user(
        &self,
        req: Request<CreateUserReq>,
    ) -> Result<Response<CreateUserResp>, Status> {
        let req = req.into_inner();
        let id = Uuid::new_v4();

        self.db
            .insert_user(id, &req.google_id)
            .await
            .map_err(CreateUserErr::Database)?;

        let response = CreateUserResp {
            user: Some(User { id: id.to_string() }),
        };

        Ok(Response::new(response))
    }

    /// Gets a user by identifier.
    ///
    /// # Errors
    /// - internal error if the user cannot be inserted into the db
    #[instrument(field(req = req.into_inner()), err)]
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
    #[instrument(field(req = req.into_inner()), err)]
    async fn get_user_id_from_google_id(
        &self,
        req: Request<GetUserIdFromGoogleIdReq>,
    ) -> Result<Response<GetUserIdFromGoogleIdResp>, Status> {
        let req = req.into_inner();
        if req.google_id.is_empty() {
            return Err(GetUserIdFromGoogleIdErr::MissingGoogleId.into());
        }
        let google_id = req.google_id;

        let id = self
            .db
            .get_user_id_from_google_id(&google_id)
            .await
            .map_err(|e| match e {
                DBError::NotFound => GetUserIdFromGoogleIdErr::NotFound,
                _ => GetUserIdFromGoogleIdErr::Database(e),
            })?;

        let response = GetUserIdFromGoogleIdResp { id: id.to_string() };
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
    #[error("missing user id")]
    MissingUserId,

    #[error("not a uuid")]
    NotAUUID,

    #[error("user not found")]
    NotFound,

    #[error("database error: {0}")]
    Database(#[from] DBError),
}

impl From<GetUserErr> for Status {
    fn from(err: GetUserErr) -> Self {
        let code = match err {
            GetUserErr::MissingUserId | GetUserErr::NotAUUID => Code::InvalidArgument,
            GetUserErr::NotFound => Code::NotFound,
            GetUserErr::Database(_) => Code::Internal,
        };
        Status::new(code, err.to_string())
    }
}

#[derive(Debug, Error)]
#[non_exhaustive]
pub enum GetUserIdFromGoogleIdErr {
    #[error("missing google id")]
    MissingGoogleId,

    #[error("user not found")]
    NotFound,

    #[error("database error: {0}")]
    Database(#[from] DBError),
}

impl From<GetUserIdFromGoogleIdErr> for Status {
    fn from(err: GetUserIdFromGoogleIdErr) -> Self {
        let code = match err {
            GetUserIdFromGoogleIdErr::MissingGoogleId => Code::InvalidArgument,
            GetUserIdFromGoogleIdErr::NotFound => Code::NotFound,
            GetUserIdFromGoogleIdErr::Database(_) => Code::Internal,
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
