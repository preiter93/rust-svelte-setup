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
