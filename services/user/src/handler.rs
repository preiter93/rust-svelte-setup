use crate::{
    db::DBClient,
    proto::{
        CreateUserReq, CreateUserResp, GetUserReq, GetUserResp, user_service_server::UserService,
    },
    utils::UuidGenerator,
};
use tonic::{Request, Response, Status};
use tracing::instrument;

#[derive(Clone)]
pub struct Handler<D, U> {
    pub db: D,
    pub uuid: U,
}

#[tonic::async_trait]
impl<D, U> UserService for Handler<D, U>
where
    D: DBClient,
    U: UuidGenerator,
{
    #[instrument(skip_all, fields(user_id), err)]
    async fn create_user(
        &self,
        req: Request<CreateUserReq>,
    ) -> Result<Response<CreateUserResp>, Status> {
        self.create_user(req).await
    }

    #[instrument(skip_all, fields(user_id), err)]
    async fn get_user(&self, req: Request<GetUserReq>) -> Result<Response<GetUserResp>, Status> {
        self.get_user(req).await
    }
}
