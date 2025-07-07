// Original file: /Users/philippreiter/Svelte/sveltekit-rust-setup/services/user/api.proto

import type * as grpc from '@grpc/grpc-js'
import type { MethodDefinition } from '@grpc/proto-loader'
import type { CreateUserReq as _proto_CreateUserReq, CreateUserReq__Output as _proto_CreateUserReq__Output } from '../proto/CreateUserReq';
import type { CreateUserResp as _proto_CreateUserResp, CreateUserResp__Output as _proto_CreateUserResp__Output } from '../proto/CreateUserResp';
import type { GetUserReq as _proto_GetUserReq, GetUserReq__Output as _proto_GetUserReq__Output } from '../proto/GetUserReq';
import type { GetUserResp as _proto_GetUserResp, GetUserResp__Output as _proto_GetUserResp__Output } from '../proto/GetUserResp';
import type { ListUsersReq as _proto_ListUsersReq, ListUsersReq__Output as _proto_ListUsersReq__Output } from '../proto/ListUsersReq';
import type { ListUsersResp as _proto_ListUsersResp, ListUsersResp__Output as _proto_ListUsersResp__Output } from '../proto/ListUsersResp';

export interface ApiServiceClient extends grpc.Client {
  CreateUser(argument: _proto_CreateUserReq, metadata: grpc.Metadata, options: grpc.CallOptions, callback: grpc.requestCallback<_proto_CreateUserResp__Output>): grpc.ClientUnaryCall;
  CreateUser(argument: _proto_CreateUserReq, metadata: grpc.Metadata, callback: grpc.requestCallback<_proto_CreateUserResp__Output>): grpc.ClientUnaryCall;
  CreateUser(argument: _proto_CreateUserReq, options: grpc.CallOptions, callback: grpc.requestCallback<_proto_CreateUserResp__Output>): grpc.ClientUnaryCall;
  CreateUser(argument: _proto_CreateUserReq, callback: grpc.requestCallback<_proto_CreateUserResp__Output>): grpc.ClientUnaryCall;
  createUser(argument: _proto_CreateUserReq, metadata: grpc.Metadata, options: grpc.CallOptions, callback: grpc.requestCallback<_proto_CreateUserResp__Output>): grpc.ClientUnaryCall;
  createUser(argument: _proto_CreateUserReq, metadata: grpc.Metadata, callback: grpc.requestCallback<_proto_CreateUserResp__Output>): grpc.ClientUnaryCall;
  createUser(argument: _proto_CreateUserReq, options: grpc.CallOptions, callback: grpc.requestCallback<_proto_CreateUserResp__Output>): grpc.ClientUnaryCall;
  createUser(argument: _proto_CreateUserReq, callback: grpc.requestCallback<_proto_CreateUserResp__Output>): grpc.ClientUnaryCall;
  
  GetUser(argument: _proto_GetUserReq, metadata: grpc.Metadata, options: grpc.CallOptions, callback: grpc.requestCallback<_proto_GetUserResp__Output>): grpc.ClientUnaryCall;
  GetUser(argument: _proto_GetUserReq, metadata: grpc.Metadata, callback: grpc.requestCallback<_proto_GetUserResp__Output>): grpc.ClientUnaryCall;
  GetUser(argument: _proto_GetUserReq, options: grpc.CallOptions, callback: grpc.requestCallback<_proto_GetUserResp__Output>): grpc.ClientUnaryCall;
  GetUser(argument: _proto_GetUserReq, callback: grpc.requestCallback<_proto_GetUserResp__Output>): grpc.ClientUnaryCall;
  getUser(argument: _proto_GetUserReq, metadata: grpc.Metadata, options: grpc.CallOptions, callback: grpc.requestCallback<_proto_GetUserResp__Output>): grpc.ClientUnaryCall;
  getUser(argument: _proto_GetUserReq, metadata: grpc.Metadata, callback: grpc.requestCallback<_proto_GetUserResp__Output>): grpc.ClientUnaryCall;
  getUser(argument: _proto_GetUserReq, options: grpc.CallOptions, callback: grpc.requestCallback<_proto_GetUserResp__Output>): grpc.ClientUnaryCall;
  getUser(argument: _proto_GetUserReq, callback: grpc.requestCallback<_proto_GetUserResp__Output>): grpc.ClientUnaryCall;
  
  ListUsers(argument: _proto_ListUsersReq, metadata: grpc.Metadata, options: grpc.CallOptions, callback: grpc.requestCallback<_proto_ListUsersResp__Output>): grpc.ClientUnaryCall;
  ListUsers(argument: _proto_ListUsersReq, metadata: grpc.Metadata, callback: grpc.requestCallback<_proto_ListUsersResp__Output>): grpc.ClientUnaryCall;
  ListUsers(argument: _proto_ListUsersReq, options: grpc.CallOptions, callback: grpc.requestCallback<_proto_ListUsersResp__Output>): grpc.ClientUnaryCall;
  ListUsers(argument: _proto_ListUsersReq, callback: grpc.requestCallback<_proto_ListUsersResp__Output>): grpc.ClientUnaryCall;
  listUsers(argument: _proto_ListUsersReq, metadata: grpc.Metadata, options: grpc.CallOptions, callback: grpc.requestCallback<_proto_ListUsersResp__Output>): grpc.ClientUnaryCall;
  listUsers(argument: _proto_ListUsersReq, metadata: grpc.Metadata, callback: grpc.requestCallback<_proto_ListUsersResp__Output>): grpc.ClientUnaryCall;
  listUsers(argument: _proto_ListUsersReq, options: grpc.CallOptions, callback: grpc.requestCallback<_proto_ListUsersResp__Output>): grpc.ClientUnaryCall;
  listUsers(argument: _proto_ListUsersReq, callback: grpc.requestCallback<_proto_ListUsersResp__Output>): grpc.ClientUnaryCall;
  
}

export interface ApiServiceHandlers extends grpc.UntypedServiceImplementation {
  CreateUser: grpc.handleUnaryCall<_proto_CreateUserReq__Output, _proto_CreateUserResp>;
  
  GetUser: grpc.handleUnaryCall<_proto_GetUserReq__Output, _proto_GetUserResp>;
  
  ListUsers: grpc.handleUnaryCall<_proto_ListUsersReq__Output, _proto_ListUsersResp>;
  
}

export interface ApiServiceDefinition extends grpc.ServiceDefinition {
  CreateUser: MethodDefinition<_proto_CreateUserReq, _proto_CreateUserResp, _proto_CreateUserReq__Output, _proto_CreateUserResp__Output>
  GetUser: MethodDefinition<_proto_GetUserReq, _proto_GetUserResp, _proto_GetUserReq__Output, _proto_GetUserResp__Output>
  ListUsers: MethodDefinition<_proto_ListUsersReq, _proto_ListUsersResp, _proto_ListUsersReq__Output, _proto_ListUsersResp__Output>
}
