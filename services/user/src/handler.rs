use std::str::FromStr;

use tonic::{Request, Response, Status};
use uuid::Uuid;

use crate::{
    db::{DBCLient, DBError},
    proto::{
        CreateUserReq, CreateUserResp, GetUserReq, GetUserResp, ListUsersReq, ListUsersResp,
        api_service_server::ApiService,
    },
    utils::internal,
};

#[derive(Clone)]
pub(crate) struct Handler {
    pub db: DBCLient,
}

#[tonic::async_trait]
impl ApiService for Handler {
    /// Creates a new user.
    ///
    /// # Errors
    /// - internal error if the user cannot be inserted into the db
    async fn create_user(
        &self,
        _: Request<CreateUserReq>,
    ) -> Result<Response<CreateUserResp>, Status> {
        let user_id = Uuid::new_v4();

        self.db.insert_user(user_id).await.map_err(internal)?;

        let response = CreateUserResp {
            id: user_id.to_string(),
        };
        Ok(Response::new(response))
    }

    /// Gets a user by id.
    ///
    /// # Errors
    /// - internal error if the user cannot be inserted into the db
    async fn get_user(&self, req: Request<GetUserReq>) -> Result<Response<GetUserResp>, Status> {
        let user_id = req.into_inner().id;
        if user_id.is_empty() {
            return Err(Status::invalid_argument("missing user id"));
        }

        let user_id = Uuid::from_str(&user_id).map_err(internal)?;
        let user = self.db.get_user(user_id).await.map_err(|e| match e {
            DBError::NotFound => Status::not_found("user not found"),
            _ => Status::internal(e.to_string()),
        })?;

        let response = GetUserResp { user: Some(user) };
        Ok(Response::new(response))
    }

    /// Lists all users.
    ///
    /// # Errors
    /// - internal error if the users cannot be retrieved from the db
    async fn list_users(
        &self,
        _: Request<ListUsersReq>,
    ) -> Result<Response<ListUsersResp>, Status> {
        let users = self.db.list_users().await.map_err(internal)?;

        let response = ListUsersResp { users };
        Ok(Response::new(response))
    }
}
