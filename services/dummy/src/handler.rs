use crate::{
    db::DBClient,
    proto::{GetEntityReq, GetEntityResp, dummy_service_server::DummyService},
};
use common::UuidGenerator;
use tonic::{Request, Response, Status};
use tracing::instrument;

#[derive(Clone)]
pub struct Handler<D, U> {
    pub db: D,
    #[allow(dead_code)]
    pub uuid: U,
}

#[tonic::async_trait]
impl<D, U> DummyService for Handler<D, U>
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
