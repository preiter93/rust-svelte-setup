use uuid::Uuid;

pub trait UuidGenerator: Send + Sync + 'static {
    fn generate(&self) -> Uuid {
        Uuid::new_v4()
    }
}

pub struct UuidV4Generator;

impl UuidGenerator for UuidV4Generator {}

#[cfg(test)]
pub mod test {
    use tonic::{Code, Response, Status};

    use crate::proto::{CreateEntityReq, Entity};

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

    pub fn fixture_uuid_string() -> String {
        fixture_uuid().to_string()
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

    pub fn fixture_create_entity_req<F>(mut func: F) -> CreateEntityReq
    where
        F: FnMut(&mut CreateEntityReq),
    {
        let mut entity = CreateEntityReq {};
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
