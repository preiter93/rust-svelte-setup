use http::Request;
use tonic::transport::Server;
use tower::{
    ServiceBuilder,
    layer::util::{Identity, Stack},
    util::MapRequestLayer,
};
use tower_http::trace::{GrpcMakeClassifier, TraceLayer};

use crate::tracing::request_span::{accept_trace, record_trace_id};

type TracerStack = Stack<TraceLayer<GrpcMakeClassifier>, Identity>;
type MiddlewareFn<B> = fn(Request<B>) -> Request<B>;

type MiddlewareStack<B> = ServiceBuilder<
    Stack<MapRequestLayer<MiddlewareFn<B>>, Stack<MapRequestLayer<MiddlewareFn<B>>, TracerStack>>,
>;

pub fn add_middleware<B, L>(router: Server<L>) -> Server<Stack<MiddlewareStack<B>, L>> {
    let service_builder: MiddlewareStack<B> = ServiceBuilder::new()
        .layer(TraceLayer::new_for_grpc())
        .map_request(accept_trace::<B> as fn(Request<B>) -> Request<B>)
        .map_request(record_trace_id::<B> as fn(Request<B>) -> Request<B>);
    let router = router.layer(service_builder);
    router
}
