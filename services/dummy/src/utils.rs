use std::str::FromStr;

use tonic::Status;
use uuid::Uuid;

use crate::error::Error;

pub fn validate_entity_id(entity_id: &str) -> Result<Uuid, Status> {
    if entity_id.is_empty() {
        return Err(Error::MissingEntityId.into());
    }

    let Ok(entity_uuid) = Uuid::from_str(entity_id) else {
        return Err(Error::InvalidEntityId(entity_id.to_string()).into());
    };

    tracing::Span::current().record("entity_id", entity_id);

    Ok(entity_uuid)
}
