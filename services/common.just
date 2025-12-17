set dotenv-load

PROTO_GEN_RS_BINARY := "../../scripts/proto-gen-rs/proto-gen-rs"
DOCKER_GEN_BINARY := "../../scripts/docker-gen/docker-gen"

generate-protos:
  @{{PROTO_GEN_RS_BINARY}}

generate-dockerfile:
  @{{DOCKER_GEN_BINARY}}

generate: generate-protos generate-dockerfile

# for local development only
create:
    psql -h localhost -U postgres -w -c "CREATE DATABASE user_db;"
    psql -h localhost -U postgres -w -c "CREATE DATABASE auth_db;"

drop:
    psql -h localhost -U postgres -w -c "DROP DATABASE IF EXISTS user_db;"
    psql -h localhost -U postgres -w -c "DROP DATABASE IF EXISTS auth_db;"

# connect:
#     psql {{DATABASE_NAME}}
