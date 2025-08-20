use uuid::Uuid;

pub trait UuidGenerator: Send + Sync + 'static {
    fn new(&self) -> Uuid;
}

pub struct UuidV4Generator;

impl UuidGenerator for UuidV4Generator {
    fn new(&self) -> Uuid {
        Uuid::new_v4()
    }
}

#[cfg(test)]
pub mod test {
    use tonic::{Code, Response, Status};

    use crate::proto::{CreateUserReq, User};

    use super::*;

    #[derive(Default)]
    pub struct MockUuidGenerator {
        pub uuid: Uuid,
    }

    impl UuidGenerator for MockUuidGenerator {
        fn new(&self) -> Uuid {
            self.uuid
        }
    }

    pub fn fixture_uuid() -> Uuid {
        Uuid::parse_str("00000000-0000-0000-0000-000000000000").unwrap()
    }

    pub fn fixture_user<F>(mut func: F) -> User
    where
        F: FnMut(&mut User),
    {
        let mut user = User {
            id: fixture_uuid().to_string(),
            name: "name".to_string(),
            email: "email".to_string(),
        };
        func(&mut user);
        user
    }

    pub fn fixture_create_user_req<F>(mut func: F) -> CreateUserReq
    where
        F: FnMut(&mut CreateUserReq),
    {
        let mut user = CreateUserReq {
            name: "name".to_string(),
            email: "email".to_string(),
            google_id: "google-id".to_string(),
        };
        func(&mut user);
        user
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
