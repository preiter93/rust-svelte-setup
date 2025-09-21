use std::str::FromStr;

use tonic::Status;
use uuid::Uuid;

use crate::error::Error;

pub trait UuidGenerator: Send + Sync + 'static {
    fn generate(&self) -> Uuid {
        Uuid::new_v4()
    }
}

pub struct UuidV4Generator;

impl UuidGenerator for UuidV4Generator {}

pub fn validate_entity_id(entity_id: &str) -> Result<Uuid, Status> {
    if entity_id.is_empty() {
        return Err(Error::MissingEntityId.into());
    }
    Uuid::from_str(entity_id).map_err(|_| Error::InvalidEntityId(entity_id.to_string()).into())
}

#[cfg(test)]
pub mod test {
    use tonic::{Code, Response, Status};

    use crate::proto::{Entity, GetEntityReq, GetEntityResp};

    use super::*;

    #[derive(Default)]
    pub struct MockUuidGenerator {
        pub uuid: Uuid,
    }

    impl UuidGenerator for MockUuidGenerator {
        fn generate(&self) -> Uuid {
            self.uuid
        }
    }

    pub fn fixture_uuid() -> Uuid {
        Uuid::parse_str("00000000-0000-0000-0000-000000000000").unwrap()
    }

    pub fn fixture_get_entity_req<F>(mut func: F) -> GetEntityReq
    where
        F: FnMut(&mut GetEntityReq),
    {
        let mut entity = GetEntityReq {
            id: fixture_uuid().to_string(),
            user_id: fixture_uuid().to_string(),
        };
        func(&mut entity);
        entity
    }

    pub fn fixture_get_entity_resp<F>(mut func: F) -> GetEntityResp
    where
        F: FnMut(&mut GetEntityResp),
    {
        let mut entity = GetEntityResp {
            entity: Some(fixture_entity(|_| {})),
        };
        func(&mut entity);
        entity
    }

    pub fn fixture_entity<F>(mut func: F) -> Entity
    where
        F: FnMut(&mut Entity),
    {
        let mut entity = Entity {
            id: fixture_uuid().to_string(),
        };
        func(&mut entity);
        entity
    }

    pub fn assert_response<T: PartialEq + std::fmt::Debug>(
        got: Result<Response<T>, Status>,
        want: Result<T, Code>,
    ) {
        match (got, want) {
            (Ok(got), Ok(want)) => assert_eq!(got.into_inner(), want),
            (Err(got), Err(want)) => assert_eq!(got.code(), want),
            (Ok(got), Err(want)) => panic!("left: {got:?}\nright: {want}"),
            (Err(got), Ok(want)) => panic!("left: {got}\nright: {want:?}"),
        }
    }
}
