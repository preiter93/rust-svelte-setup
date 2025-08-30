// Original file: /Users/philippreiter/Svelte/rust-svelte-setup/services/auth/api.proto

import type * as grpc from '@grpc/grpc-js'
import type { MethodDefinition } from '@grpc/proto-loader'
import type { CreateSessionReq as _proto_CreateSessionReq, CreateSessionReq__Output as _proto_CreateSessionReq__Output } from '../proto/CreateSessionReq';
import type { CreateSessionResp as _proto_CreateSessionResp, CreateSessionResp__Output as _proto_CreateSessionResp__Output } from '../proto/CreateSessionResp';
import type { DeleteSessionReq as _proto_DeleteSessionReq, DeleteSessionReq__Output as _proto_DeleteSessionReq__Output } from '../proto/DeleteSessionReq';
import type { DeleteSessionResp as _proto_DeleteSessionResp, DeleteSessionResp__Output as _proto_DeleteSessionResp__Output } from '../proto/DeleteSessionResp';
import type { HandleOauthCallbackReq as _proto_HandleOauthCallbackReq, HandleOauthCallbackReq__Output as _proto_HandleOauthCallbackReq__Output } from '../proto/HandleOauthCallbackReq';
import type { HandleOauthCallbackResp as _proto_HandleOauthCallbackResp, HandleOauthCallbackResp__Output as _proto_HandleOauthCallbackResp__Output } from '../proto/HandleOauthCallbackResp';
import type { LinkOauthTokenReq as _proto_LinkOauthTokenReq, LinkOauthTokenReq__Output as _proto_LinkOauthTokenReq__Output } from '../proto/LinkOauthTokenReq';
import type { LinkOauthTokenResp as _proto_LinkOauthTokenResp, LinkOauthTokenResp__Output as _proto_LinkOauthTokenResp__Output } from '../proto/LinkOauthTokenResp';
import type { StartOauthLoginReq as _proto_StartOauthLoginReq, StartOauthLoginReq__Output as _proto_StartOauthLoginReq__Output } from '../proto/StartOauthLoginReq';
import type { StartOauthLoginResp as _proto_StartOauthLoginResp, StartOauthLoginResp__Output as _proto_StartOauthLoginResp__Output } from '../proto/StartOauthLoginResp';
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
  
  DeleteSession(argument: _proto_DeleteSessionReq, metadata: grpc.Metadata, options: grpc.CallOptions, callback: grpc.requestCallback<_proto_DeleteSessionResp__Output>): grpc.ClientUnaryCall;
  DeleteSession(argument: _proto_DeleteSessionReq, metadata: grpc.Metadata, callback: grpc.requestCallback<_proto_DeleteSessionResp__Output>): grpc.ClientUnaryCall;
  DeleteSession(argument: _proto_DeleteSessionReq, options: grpc.CallOptions, callback: grpc.requestCallback<_proto_DeleteSessionResp__Output>): grpc.ClientUnaryCall;
  DeleteSession(argument: _proto_DeleteSessionReq, callback: grpc.requestCallback<_proto_DeleteSessionResp__Output>): grpc.ClientUnaryCall;
  deleteSession(argument: _proto_DeleteSessionReq, metadata: grpc.Metadata, options: grpc.CallOptions, callback: grpc.requestCallback<_proto_DeleteSessionResp__Output>): grpc.ClientUnaryCall;
  deleteSession(argument: _proto_DeleteSessionReq, metadata: grpc.Metadata, callback: grpc.requestCallback<_proto_DeleteSessionResp__Output>): grpc.ClientUnaryCall;
  deleteSession(argument: _proto_DeleteSessionReq, options: grpc.CallOptions, callback: grpc.requestCallback<_proto_DeleteSessionResp__Output>): grpc.ClientUnaryCall;
  deleteSession(argument: _proto_DeleteSessionReq, callback: grpc.requestCallback<_proto_DeleteSessionResp__Output>): grpc.ClientUnaryCall;
  
  HandleOauthCallback(argument: _proto_HandleOauthCallbackReq, metadata: grpc.Metadata, options: grpc.CallOptions, callback: grpc.requestCallback<_proto_HandleOauthCallbackResp__Output>): grpc.ClientUnaryCall;
  HandleOauthCallback(argument: _proto_HandleOauthCallbackReq, metadata: grpc.Metadata, callback: grpc.requestCallback<_proto_HandleOauthCallbackResp__Output>): grpc.ClientUnaryCall;
  HandleOauthCallback(argument: _proto_HandleOauthCallbackReq, options: grpc.CallOptions, callback: grpc.requestCallback<_proto_HandleOauthCallbackResp__Output>): grpc.ClientUnaryCall;
  HandleOauthCallback(argument: _proto_HandleOauthCallbackReq, callback: grpc.requestCallback<_proto_HandleOauthCallbackResp__Output>): grpc.ClientUnaryCall;
  handleOauthCallback(argument: _proto_HandleOauthCallbackReq, metadata: grpc.Metadata, options: grpc.CallOptions, callback: grpc.requestCallback<_proto_HandleOauthCallbackResp__Output>): grpc.ClientUnaryCall;
  handleOauthCallback(argument: _proto_HandleOauthCallbackReq, metadata: grpc.Metadata, callback: grpc.requestCallback<_proto_HandleOauthCallbackResp__Output>): grpc.ClientUnaryCall;
  handleOauthCallback(argument: _proto_HandleOauthCallbackReq, options: grpc.CallOptions, callback: grpc.requestCallback<_proto_HandleOauthCallbackResp__Output>): grpc.ClientUnaryCall;
  handleOauthCallback(argument: _proto_HandleOauthCallbackReq, callback: grpc.requestCallback<_proto_HandleOauthCallbackResp__Output>): grpc.ClientUnaryCall;
  
  LinkOauthToken(argument: _proto_LinkOauthTokenReq, metadata: grpc.Metadata, options: grpc.CallOptions, callback: grpc.requestCallback<_proto_LinkOauthTokenResp__Output>): grpc.ClientUnaryCall;
  LinkOauthToken(argument: _proto_LinkOauthTokenReq, metadata: grpc.Metadata, callback: grpc.requestCallback<_proto_LinkOauthTokenResp__Output>): grpc.ClientUnaryCall;
  LinkOauthToken(argument: _proto_LinkOauthTokenReq, options: grpc.CallOptions, callback: grpc.requestCallback<_proto_LinkOauthTokenResp__Output>): grpc.ClientUnaryCall;
  LinkOauthToken(argument: _proto_LinkOauthTokenReq, callback: grpc.requestCallback<_proto_LinkOauthTokenResp__Output>): grpc.ClientUnaryCall;
  linkOauthToken(argument: _proto_LinkOauthTokenReq, metadata: grpc.Metadata, options: grpc.CallOptions, callback: grpc.requestCallback<_proto_LinkOauthTokenResp__Output>): grpc.ClientUnaryCall;
  linkOauthToken(argument: _proto_LinkOauthTokenReq, metadata: grpc.Metadata, callback: grpc.requestCallback<_proto_LinkOauthTokenResp__Output>): grpc.ClientUnaryCall;
  linkOauthToken(argument: _proto_LinkOauthTokenReq, options: grpc.CallOptions, callback: grpc.requestCallback<_proto_LinkOauthTokenResp__Output>): grpc.ClientUnaryCall;
  linkOauthToken(argument: _proto_LinkOauthTokenReq, callback: grpc.requestCallback<_proto_LinkOauthTokenResp__Output>): grpc.ClientUnaryCall;
  
  StartOauthLogin(argument: _proto_StartOauthLoginReq, metadata: grpc.Metadata, options: grpc.CallOptions, callback: grpc.requestCallback<_proto_StartOauthLoginResp__Output>): grpc.ClientUnaryCall;
  StartOauthLogin(argument: _proto_StartOauthLoginReq, metadata: grpc.Metadata, callback: grpc.requestCallback<_proto_StartOauthLoginResp__Output>): grpc.ClientUnaryCall;
  StartOauthLogin(argument: _proto_StartOauthLoginReq, options: grpc.CallOptions, callback: grpc.requestCallback<_proto_StartOauthLoginResp__Output>): grpc.ClientUnaryCall;
  StartOauthLogin(argument: _proto_StartOauthLoginReq, callback: grpc.requestCallback<_proto_StartOauthLoginResp__Output>): grpc.ClientUnaryCall;
  startOauthLogin(argument: _proto_StartOauthLoginReq, metadata: grpc.Metadata, options: grpc.CallOptions, callback: grpc.requestCallback<_proto_StartOauthLoginResp__Output>): grpc.ClientUnaryCall;
  startOauthLogin(argument: _proto_StartOauthLoginReq, metadata: grpc.Metadata, callback: grpc.requestCallback<_proto_StartOauthLoginResp__Output>): grpc.ClientUnaryCall;
  startOauthLogin(argument: _proto_StartOauthLoginReq, options: grpc.CallOptions, callback: grpc.requestCallback<_proto_StartOauthLoginResp__Output>): grpc.ClientUnaryCall;
  startOauthLogin(argument: _proto_StartOauthLoginReq, callback: grpc.requestCallback<_proto_StartOauthLoginResp__Output>): grpc.ClientUnaryCall;
  
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
  
  DeleteSession: grpc.handleUnaryCall<_proto_DeleteSessionReq__Output, _proto_DeleteSessionResp>;
  
  HandleOauthCallback: grpc.handleUnaryCall<_proto_HandleOauthCallbackReq__Output, _proto_HandleOauthCallbackResp>;
  
  LinkOauthToken: grpc.handleUnaryCall<_proto_LinkOauthTokenReq__Output, _proto_LinkOauthTokenResp>;
  
  StartOauthLogin: grpc.handleUnaryCall<_proto_StartOauthLoginReq__Output, _proto_StartOauthLoginResp>;
  
  ValidateSession: grpc.handleUnaryCall<_proto_ValidateSessionReq__Output, _proto_ValidateSessionResp>;
  
}

export interface ApiServiceDefinition extends grpc.ServiceDefinition {
  CreateSession: MethodDefinition<_proto_CreateSessionReq, _proto_CreateSessionResp, _proto_CreateSessionReq__Output, _proto_CreateSessionResp__Output>
  DeleteSession: MethodDefinition<_proto_DeleteSessionReq, _proto_DeleteSessionResp, _proto_DeleteSessionReq__Output, _proto_DeleteSessionResp__Output>
  HandleOauthCallback: MethodDefinition<_proto_HandleOauthCallbackReq, _proto_HandleOauthCallbackResp, _proto_HandleOauthCallbackReq__Output, _proto_HandleOauthCallbackResp__Output>
  LinkOauthToken: MethodDefinition<_proto_LinkOauthTokenReq, _proto_LinkOauthTokenResp, _proto_LinkOauthTokenReq__Output, _proto_LinkOauthTokenResp__Output>
  StartOauthLogin: MethodDefinition<_proto_StartOauthLoginReq, _proto_StartOauthLoginResp, _proto_StartOauthLoginReq__Output, _proto_StartOauthLoginResp__Output>
  ValidateSession: MethodDefinition<_proto_ValidateSessionReq, _proto_ValidateSessionResp, _proto_ValidateSessionReq__Output, _proto_ValidateSessionResp__Output>
}
