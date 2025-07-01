import type * as grpc from '@grpc/grpc-js';
import type { MessageTypeDefinition } from '@grpc/proto-loader';

import type { ApiServiceClient as _proto_ApiServiceClient, ApiServiceDefinition as _proto_ApiServiceDefinition } from './proto/ApiService';

type SubtypeConstructor<Constructor extends new (...args: any) => any, Subtype> = {
  new(...args: ConstructorParameters<Constructor>): Subtype;
};

export interface ProtoGrpcType {
  proto: {
    ApiService: SubtypeConstructor<typeof grpc.Client, _proto_ApiServiceClient> & { service: _proto_ApiServiceDefinition }
    CreateUserReq: MessageTypeDefinition
    CreateUserResp: MessageTypeDefinition
    GetUserReq: MessageTypeDefinition
    GetUserResp: MessageTypeDefinition
    ListUsersReq: MessageTypeDefinition
    ListUsersResp: MessageTypeDefinition
    User: MessageTypeDefinition
  }
}

