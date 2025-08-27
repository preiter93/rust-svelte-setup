// Original file: /Users/philippreiter/Svelte/rust-svelte-setup/services/auth/api.proto

import type * as grpc from '@grpc/grpc-js'
import type { MethodDefinition } from '@grpc/proto-loader'
import type { CreateSessionReq as _proto_CreateSessionReq, CreateSessionReq__Output as _proto_CreateSessionReq__Output } from '../proto/CreateSessionReq';
import type { CreateSessionResp as _proto_CreateSessionResp, CreateSessionResp__Output as _proto_CreateSessionResp__Output } from '../proto/CreateSessionResp';
import type { DeleteSessionReq as _proto_DeleteSessionReq, DeleteSessionReq__Output as _proto_DeleteSessionReq__Output } from '../proto/DeleteSessionReq';
import type { DeleteSessionResp as _proto_DeleteSessionResp, DeleteSessionResp__Output as _proto_DeleteSessionResp__Output } from '../proto/DeleteSessionResp';
import type { HandleGithubCallbackReq as _proto_HandleGithubCallbackReq, HandleGithubCallbackReq__Output as _proto_HandleGithubCallbackReq__Output } from '../proto/HandleGithubCallbackReq';
import type { HandleGithubCallbackResp as _proto_HandleGithubCallbackResp, HandleGithubCallbackResp__Output as _proto_HandleGithubCallbackResp__Output } from '../proto/HandleGithubCallbackResp';
import type { HandleGoogleCallbackReq as _proto_HandleGoogleCallbackReq, HandleGoogleCallbackReq__Output as _proto_HandleGoogleCallbackReq__Output } from '../proto/HandleGoogleCallbackReq';
import type { HandleGoogleCallbackResp as _proto_HandleGoogleCallbackResp, HandleGoogleCallbackResp__Output as _proto_HandleGoogleCallbackResp__Output } from '../proto/HandleGoogleCallbackResp';
import type { StartGithubLoginReq as _proto_StartGithubLoginReq, StartGithubLoginReq__Output as _proto_StartGithubLoginReq__Output } from '../proto/StartGithubLoginReq';
import type { StartGithubLoginResp as _proto_StartGithubLoginResp, StartGithubLoginResp__Output as _proto_StartGithubLoginResp__Output } from '../proto/StartGithubLoginResp';
import type { StartGoogleLoginReq as _proto_StartGoogleLoginReq, StartGoogleLoginReq__Output as _proto_StartGoogleLoginReq__Output } from '../proto/StartGoogleLoginReq';
import type { StartGoogleLoginResp as _proto_StartGoogleLoginResp, StartGoogleLoginResp__Output as _proto_StartGoogleLoginResp__Output } from '../proto/StartGoogleLoginResp';
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
  
  HandleGithubCallback(argument: _proto_HandleGithubCallbackReq, metadata: grpc.Metadata, options: grpc.CallOptions, callback: grpc.requestCallback<_proto_HandleGithubCallbackResp__Output>): grpc.ClientUnaryCall;
  HandleGithubCallback(argument: _proto_HandleGithubCallbackReq, metadata: grpc.Metadata, callback: grpc.requestCallback<_proto_HandleGithubCallbackResp__Output>): grpc.ClientUnaryCall;
  HandleGithubCallback(argument: _proto_HandleGithubCallbackReq, options: grpc.CallOptions, callback: grpc.requestCallback<_proto_HandleGithubCallbackResp__Output>): grpc.ClientUnaryCall;
  HandleGithubCallback(argument: _proto_HandleGithubCallbackReq, callback: grpc.requestCallback<_proto_HandleGithubCallbackResp__Output>): grpc.ClientUnaryCall;
  handleGithubCallback(argument: _proto_HandleGithubCallbackReq, metadata: grpc.Metadata, options: grpc.CallOptions, callback: grpc.requestCallback<_proto_HandleGithubCallbackResp__Output>): grpc.ClientUnaryCall;
  handleGithubCallback(argument: _proto_HandleGithubCallbackReq, metadata: grpc.Metadata, callback: grpc.requestCallback<_proto_HandleGithubCallbackResp__Output>): grpc.ClientUnaryCall;
  handleGithubCallback(argument: _proto_HandleGithubCallbackReq, options: grpc.CallOptions, callback: grpc.requestCallback<_proto_HandleGithubCallbackResp__Output>): grpc.ClientUnaryCall;
  handleGithubCallback(argument: _proto_HandleGithubCallbackReq, callback: grpc.requestCallback<_proto_HandleGithubCallbackResp__Output>): grpc.ClientUnaryCall;
  
  HandleGoogleCallback(argument: _proto_HandleGoogleCallbackReq, metadata: grpc.Metadata, options: grpc.CallOptions, callback: grpc.requestCallback<_proto_HandleGoogleCallbackResp__Output>): grpc.ClientUnaryCall;
  HandleGoogleCallback(argument: _proto_HandleGoogleCallbackReq, metadata: grpc.Metadata, callback: grpc.requestCallback<_proto_HandleGoogleCallbackResp__Output>): grpc.ClientUnaryCall;
  HandleGoogleCallback(argument: _proto_HandleGoogleCallbackReq, options: grpc.CallOptions, callback: grpc.requestCallback<_proto_HandleGoogleCallbackResp__Output>): grpc.ClientUnaryCall;
  HandleGoogleCallback(argument: _proto_HandleGoogleCallbackReq, callback: grpc.requestCallback<_proto_HandleGoogleCallbackResp__Output>): grpc.ClientUnaryCall;
  handleGoogleCallback(argument: _proto_HandleGoogleCallbackReq, metadata: grpc.Metadata, options: grpc.CallOptions, callback: grpc.requestCallback<_proto_HandleGoogleCallbackResp__Output>): grpc.ClientUnaryCall;
  handleGoogleCallback(argument: _proto_HandleGoogleCallbackReq, metadata: grpc.Metadata, callback: grpc.requestCallback<_proto_HandleGoogleCallbackResp__Output>): grpc.ClientUnaryCall;
  handleGoogleCallback(argument: _proto_HandleGoogleCallbackReq, options: grpc.CallOptions, callback: grpc.requestCallback<_proto_HandleGoogleCallbackResp__Output>): grpc.ClientUnaryCall;
  handleGoogleCallback(argument: _proto_HandleGoogleCallbackReq, callback: grpc.requestCallback<_proto_HandleGoogleCallbackResp__Output>): grpc.ClientUnaryCall;
  
  StartGithubLogin(argument: _proto_StartGithubLoginReq, metadata: grpc.Metadata, options: grpc.CallOptions, callback: grpc.requestCallback<_proto_StartGithubLoginResp__Output>): grpc.ClientUnaryCall;
  StartGithubLogin(argument: _proto_StartGithubLoginReq, metadata: grpc.Metadata, callback: grpc.requestCallback<_proto_StartGithubLoginResp__Output>): grpc.ClientUnaryCall;
  StartGithubLogin(argument: _proto_StartGithubLoginReq, options: grpc.CallOptions, callback: grpc.requestCallback<_proto_StartGithubLoginResp__Output>): grpc.ClientUnaryCall;
  StartGithubLogin(argument: _proto_StartGithubLoginReq, callback: grpc.requestCallback<_proto_StartGithubLoginResp__Output>): grpc.ClientUnaryCall;
  startGithubLogin(argument: _proto_StartGithubLoginReq, metadata: grpc.Metadata, options: grpc.CallOptions, callback: grpc.requestCallback<_proto_StartGithubLoginResp__Output>): grpc.ClientUnaryCall;
  startGithubLogin(argument: _proto_StartGithubLoginReq, metadata: grpc.Metadata, callback: grpc.requestCallback<_proto_StartGithubLoginResp__Output>): grpc.ClientUnaryCall;
  startGithubLogin(argument: _proto_StartGithubLoginReq, options: grpc.CallOptions, callback: grpc.requestCallback<_proto_StartGithubLoginResp__Output>): grpc.ClientUnaryCall;
  startGithubLogin(argument: _proto_StartGithubLoginReq, callback: grpc.requestCallback<_proto_StartGithubLoginResp__Output>): grpc.ClientUnaryCall;
  
  StartGoogleLogin(argument: _proto_StartGoogleLoginReq, metadata: grpc.Metadata, options: grpc.CallOptions, callback: grpc.requestCallback<_proto_StartGoogleLoginResp__Output>): grpc.ClientUnaryCall;
  StartGoogleLogin(argument: _proto_StartGoogleLoginReq, metadata: grpc.Metadata, callback: grpc.requestCallback<_proto_StartGoogleLoginResp__Output>): grpc.ClientUnaryCall;
  StartGoogleLogin(argument: _proto_StartGoogleLoginReq, options: grpc.CallOptions, callback: grpc.requestCallback<_proto_StartGoogleLoginResp__Output>): grpc.ClientUnaryCall;
  StartGoogleLogin(argument: _proto_StartGoogleLoginReq, callback: grpc.requestCallback<_proto_StartGoogleLoginResp__Output>): grpc.ClientUnaryCall;
  startGoogleLogin(argument: _proto_StartGoogleLoginReq, metadata: grpc.Metadata, options: grpc.CallOptions, callback: grpc.requestCallback<_proto_StartGoogleLoginResp__Output>): grpc.ClientUnaryCall;
  startGoogleLogin(argument: _proto_StartGoogleLoginReq, metadata: grpc.Metadata, callback: grpc.requestCallback<_proto_StartGoogleLoginResp__Output>): grpc.ClientUnaryCall;
  startGoogleLogin(argument: _proto_StartGoogleLoginReq, options: grpc.CallOptions, callback: grpc.requestCallback<_proto_StartGoogleLoginResp__Output>): grpc.ClientUnaryCall;
  startGoogleLogin(argument: _proto_StartGoogleLoginReq, callback: grpc.requestCallback<_proto_StartGoogleLoginResp__Output>): grpc.ClientUnaryCall;
  
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
  
  HandleGithubCallback: grpc.handleUnaryCall<_proto_HandleGithubCallbackReq__Output, _proto_HandleGithubCallbackResp>;
  
  HandleGoogleCallback: grpc.handleUnaryCall<_proto_HandleGoogleCallbackReq__Output, _proto_HandleGoogleCallbackResp>;
  
  StartGithubLogin: grpc.handleUnaryCall<_proto_StartGithubLoginReq__Output, _proto_StartGithubLoginResp>;
  
  StartGoogleLogin: grpc.handleUnaryCall<_proto_StartGoogleLoginReq__Output, _proto_StartGoogleLoginResp>;
  
  ValidateSession: grpc.handleUnaryCall<_proto_ValidateSessionReq__Output, _proto_ValidateSessionResp>;
  
}

export interface ApiServiceDefinition extends grpc.ServiceDefinition {
  CreateSession: MethodDefinition<_proto_CreateSessionReq, _proto_CreateSessionResp, _proto_CreateSessionReq__Output, _proto_CreateSessionResp__Output>
  DeleteSession: MethodDefinition<_proto_DeleteSessionReq, _proto_DeleteSessionResp, _proto_DeleteSessionReq__Output, _proto_DeleteSessionResp__Output>
  HandleGithubCallback: MethodDefinition<_proto_HandleGithubCallbackReq, _proto_HandleGithubCallbackResp, _proto_HandleGithubCallbackReq__Output, _proto_HandleGithubCallbackResp__Output>
  HandleGoogleCallback: MethodDefinition<_proto_HandleGoogleCallbackReq, _proto_HandleGoogleCallbackResp, _proto_HandleGoogleCallbackReq__Output, _proto_HandleGoogleCallbackResp__Output>
  StartGithubLogin: MethodDefinition<_proto_StartGithubLoginReq, _proto_StartGithubLoginResp, _proto_StartGithubLoginReq__Output, _proto_StartGithubLoginResp__Output>
  StartGoogleLogin: MethodDefinition<_proto_StartGoogleLoginReq, _proto_StartGoogleLoginResp, _proto_StartGoogleLoginReq__Output, _proto_StartGoogleLoginResp__Output>
  ValidateSession: MethodDefinition<_proto_ValidateSessionReq, _proto_ValidateSessionResp, _proto_ValidateSessionReq__Output, _proto_ValidateSessionResp__Output>
}
