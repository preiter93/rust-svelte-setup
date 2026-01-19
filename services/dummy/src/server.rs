use crate::{
    db::DBClient,
    proto::{GetEntityReq, GetEntityResp, api_service_server::ApiService},
};
use common::UuidGenerator;
use tonic::{Request, Response, Status};
use tracing::instrument;

#[derive(Clone)]
pub struct Server<D, U> {
    pub db: D,
    #[allow(dead_code)]
    pub uuid: U,
}

#[tonic::async_trait]
impl<D, U> ApiService for Server<D, U>
where
    D: DBClient,
    U: UuidGenerator,
{
    #[instrument(skip_all, fields(user_id), err)]
    async fn get_entity(
        &self,
        req: Request<GetEntityReq>,
    ) -> Result<Response<GetEntityResp>, Status> {
        self.get_entity(req).await
    }
}
