use tonic::Status;

#[allow(clippy::needless_pass_by_value)]
pub(crate) fn internal<E: ToString>(e: E) -> Status {
    Status::internal(e.to_string())
}
