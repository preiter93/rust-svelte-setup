#![cfg(test)]

use uuid::Uuid;

use crate::proto::{CreateUserReq, User};

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
    };
    func(&mut user);
    user
}

#[derive(Clone)]
pub struct DBUser {
    pub id: Uuid,
    pub name: &'static str,
    pub email: &'static str,
}

pub fn fixture_db_user<F>(mut func: F) -> DBUser
where
    F: FnMut(&mut DBUser),
{
    let mut user = DBUser {
        id: fixture_uuid(),
        name: "name",
        email: "email",
    };
    func(&mut user);
    user
}
