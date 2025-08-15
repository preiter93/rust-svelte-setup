use std::str::FromStr;

use crate::{
    db::DBCLient,
    error::{CreateUserErr, DBError, GetUserErr, GetUserIdFromGoogleIdErr},
    proto::{
        CreateUserReq, CreateUserResp, GetUserIdFromGoogleIdReq, GetUserIdFromGoogleIdResp,
        GetUserReq, GetUserResp, User, api_service_server::ApiService,
    },
};
use tonic::{Request, Response, Status};
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

        let name = req.name;
        if name.is_empty() {
            return Err(CreateUserErr::MissingName.into());
        }

        let email = req.email;
        if email.is_empty() {
            return Err(CreateUserErr::MissingEmail.into());
        }

        let google_id = req.google_id;

        self.db
            .insert_user(id, &name, &email, &google_id)
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
