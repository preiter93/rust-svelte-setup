#[cfg(test)]
pub mod test {
    use crate::proto::{CreateUserReq, User};
    use crate::{db::DBClient, utils::UuidGenerator};
    use tokio::sync::Mutex;
    use tonic::{Code, Response, Status, async_trait};
    use uuid::Uuid;

    use crate::error::DBError;

    pub struct MockDBClient {
        pub get_user: Mutex<Option<Result<User, DBError>>>,
        pub insert_user: Mutex<Option<Result<(), DBError>>>,
        pub get_user_id_from_google_id: Mutex<Option<Result<Uuid, DBError>>>,
    }
    impl Default for MockDBClient {
        fn default() -> Self {
            Self {
                insert_user: Mutex::new(None),
                get_user: Mutex::new(None),
                get_user_id_from_google_id: Mutex::new(None),
            }
        }
    }

    #[async_trait]
    impl DBClient for MockDBClient {
        async fn insert_user(&self, _: Uuid, _: &str, _: &str, _: &str) -> Result<(), DBError> {
            self.insert_user.lock().await.take().unwrap()
        }

        async fn get_user(&self, _: Uuid) -> Result<User, DBError> {
            self.get_user.lock().await.take().unwrap()
        }

        async fn get_user_id_from_google_id(&self, _: &str) -> Result<Uuid, DBError> {
            self.get_user_id_from_google_id.lock().await.take().unwrap()
        }
    }

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
