// Original file: /Users/philippreiter/Svelte/sveltekit-rust-setup/services/auth/api.proto

import type * as grpc from '@grpc/grpc-js'
import type { MethodDefinition } from '@grpc/proto-loader'
import type { CreateSessionReq as _proto_CreateSessionReq, CreateSessionReq__Output as _proto_CreateSessionReq__Output } from '../proto/CreateSessionReq';
import type { CreateSessionResp as _proto_CreateSessionResp, CreateSessionResp__Output as _proto_CreateSessionResp__Output } from '../proto/CreateSessionResp';
import type { ValidateSessionReq as _proto_ValidateSessionReq, ValidateSessionReq__Output as _proto_ValidateSessionReq__Output } from '../proto/ValidateSessionReq';
import type { ValidateSessionResp as _proto_ValidateSessionResp, ValidateSessionResp__Output as _proto_ValidateSessionResp__Output } from '../proto/ValidateSessionResp';

export interface ApiServiceClient extends grpc.Client {
  CreateSession(argument: _proto_CreateSessionReq, metadata: grpc.Metadata, options: grpc.CallOptions, callback: grpc.requestCallback<_proto_CreateSessionResp__Output>): grpc.ClientUnaryCall;
  CreateSession(argument: _proto_CreateSessionReq, metadata: grpc.Metadata, callback: grpc.requestCallback<_proto_CreateSessionResp__Output>): grpc.ClientUnaryCall;
  CreateSession(argument: _proto_CreateSessionReq, options: grpc.CallOptions, callback: grpc.requestCallback<_proto_CreateSessionResp__Output>): grpc.ClientUnaryCall;
  CreateSession(argument: _proto_CreateSessionReq, callback: grpc.requestCallback<_proto_CreateSessionResp__Output>): grpc.ClientUnaryCall;
  createSession(argument: _proto_CreateSessionReq, metadata: grpc.Metadata, options: grpc.CallOptions, callback: grpc.requestCallback<_proto_CreateSessionResp__Output>): grpc.ClientUnaryCall;
  createSession(argument: _proto_CreateSessionReq, metadata: grpc.Metadata, callback: grpc.requestCallback<_proto_CreateSessionResp__Output>): grpc.ClientUnaryCall;
  createSession(argument: _proto_CreateSessionReq, options: grpc.CallOptions, callback: grpc.requestCallback<_proto_CreateSessionResp__Output>): grpc.ClientUnaryCall;
  createSession(argument: _proto_CreateSessionReq, callback: grpc.requestCallback<_proto_CreateSessionResp__Output>): grpc.ClientUnaryCall;
  
  ValidateSession(argument: _proto_ValidateSessionReq, metadata: grpc.Metadata, options: grpc.CallOptions, callback: grpc.requestCallback<_proto_ValidateSessionResp__Output>): grpc.ClientUnaryCall;
  ValidateSession(argument: _proto_ValidateSessionReq, metadata: grpc.Metadata, callback: grpc.requestCallback<_proto_ValidateSessionResp__Output>): grpc.ClientUnaryCall;
  ValidateSession(argument: _proto_ValidateSessionReq, options: grpc.CallOptions, callback: grpc.requestCallback<_proto_ValidateSessionResp__Output>): grpc.ClientUnaryCall;
  ValidateSession(argument: _proto_ValidateSessionReq, callback: grpc.requestCallback<_proto_ValidateSessionResp__Output>): grpc.ClientUnaryCall;
  validateSession(argument: _proto_ValidateSessionReq, metadata: grpc.Metadata, options: grpc.CallOptions, callback: grpc.requestCallback<_proto_ValidateSessionResp__Output>): grpc.ClientUnaryCall;
  validateSession(argument: _proto_ValidateSessionReq, metadata: grpc.Metadata, callback: grpc.requestCallback<_proto_ValidateSessionResp__Output>): grpc.ClientUnaryCall;
  validateSession(argument: _proto_ValidateSessionReq, options: grpc.CallOptions, callback: grpc.requestCallback<_proto_ValidateSessionResp__Output>): grpc.ClientUnaryCall;
  validateSession(argument: _proto_ValidateSessionReq, callback: grpc.requestCallback<_proto_ValidateSessionResp__Output>): grpc.ClientUnaryCall;
  
}

export interface ApiServiceHandlers extends grpc.UntypedServiceImplementation {
  CreateSession: grpc.handleUnaryCall<_proto_CreateSessionReq__Output, _proto_CreateSessionResp>;
  
  ValidateSession: grpc.handleUnaryCall<_proto_ValidateSessionReq__Output, _proto_ValidateSessionResp>;
  
}

export interface ApiServiceDefinition extends grpc.ServiceDefinition {
  CreateSession: MethodDefinition<_proto_CreateSessionReq, _proto_CreateSessionResp, _proto_CreateSessionReq__Output, _proto_CreateSessionResp__Output>
  ValidateSession: MethodDefinition<_proto_ValidateSessionReq, _proto_ValidateSessionResp, _proto_ValidateSessionReq__Output, _proto_ValidateSessionResp__Output>
}
