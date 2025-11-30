// Original file: /Users/philippreiter/Svelte/rust-svelte-setup/services/dummy/api.proto

import type * as grpc from '@grpc/grpc-js'
import type { MethodDefinition } from '@grpc/proto-loader'
import type { GetEntityReq as _proto_GetEntityReq, GetEntityReq__Output as _proto_GetEntityReq__Output } from '../proto/GetEntityReq';
import type { GetEntityResp as _proto_GetEntityResp, GetEntityResp__Output as _proto_GetEntityResp__Output } from '../proto/GetEntityResp';

export interface ApiServiceClient extends grpc.Client {
  GetEntity(argument: _proto_GetEntityReq, metadata: grpc.Metadata, options: grpc.CallOptions, callback: grpc.requestCallback<_proto_GetEntityResp__Output>): grpc.ClientUnaryCall;
  GetEntity(argument: _proto_GetEntityReq, metadata: grpc.Metadata, callback: grpc.requestCallback<_proto_GetEntityResp__Output>): grpc.ClientUnaryCall;
  GetEntity(argument: _proto_GetEntityReq, options: grpc.CallOptions, callback: grpc.requestCallback<_proto_GetEntityResp__Output>): grpc.ClientUnaryCall;
  GetEntity(argument: _proto_GetEntityReq, callback: grpc.requestCallback<_proto_GetEntityResp__Output>): grpc.ClientUnaryCall;
  getEntity(argument: _proto_GetEntityReq, metadata: grpc.Metadata, options: grpc.CallOptions, callback: grpc.requestCallback<_proto_GetEntityResp__Output>): grpc.ClientUnaryCall;
  getEntity(argument: _proto_GetEntityReq, metadata: grpc.Metadata, callback: grpc.requestCallback<_proto_GetEntityResp__Output>): grpc.ClientUnaryCall;
  getEntity(argument: _proto_GetEntityReq, options: grpc.CallOptions, callback: grpc.requestCallback<_proto_GetEntityResp__Output>): grpc.ClientUnaryCall;
  getEntity(argument: _proto_GetEntityReq, callback: grpc.requestCallback<_proto_GetEntityResp__Output>): grpc.ClientUnaryCall;
  
}

export interface ApiServiceHandlers extends grpc.UntypedServiceImplementation {
  GetEntity: grpc.handleUnaryCall<_proto_GetEntityReq__Output, _proto_GetEntityResp>;
  
}

export interface ApiServiceDefinition extends grpc.ServiceDefinition {
  GetEntity: MethodDefinition<_proto_GetEntityReq, _proto_GetEntityResp, _proto_GetEntityReq__Output, _proto_GetEntityResp__Output>
}
