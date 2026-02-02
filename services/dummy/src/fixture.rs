#![cfg(test)]

use uuid::Uuid;

use crate::proto::{Entity, GetEntityReq, GetEntityResp};

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

#[derive(Clone)]
pub struct DBEntity {
    pub id: Uuid,
    pub user_id: Uuid,
}

pub fn fixture_db_entity<F>(mut func: F) -> DBEntity
where
    F: FnMut(&mut DBEntity),
{
    let mut entity = DBEntity {
        id: fixture_uuid(),
        user_id: fixture_uuid(),
    };
    func(&mut entity);
    entity
}
