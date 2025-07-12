set dotenv-load

# export PG_HOST := if env_var_or_default("LOCAL", "false") == "true" {
#     "${PG_HOST_LOCAL}"
# } else {
#     "${PG_HOST_REMOTE}"
# }

# DATABASE_URL := "postgres://$PG_USER:$PG_PASSWORD@$PG_HOST_LOCAL:$PG_PORT/$PG_DBNAME?sslmode=disable"

DATABASE_NAME := "$PG_DBNAME"

# MIGRATIONS_DIR := "migrations"

PROTO_GEN_RS_BINARY := "../../scripts/proto-gen-rs/proto-gen-rs"

generate-protos:
  {{PROTO_GEN_RS_BINARY}}


# migrate-up:
#     @echo "Migrate"
#     @echo {{DATABASE_URL}}
#     migrate -path {{MIGRATIONS_DIR}} -database "{{DATABASE_URL}}" up
#
# migrate-down:
#     migrate -path {{MIGRATIONS_DIR}} -database "{{DATABASE_URL}}" down
#
# create-migration name:
#     migrate create -ext sql -dir {{MIGRATIONS_DIR}} -seq {{name}}

# for local development
create:
    psql -h localhost -U postgres -w -c "CREATE DATABASE {{DATABASE_NAME}};"

drop:
    psql -h localhost -U postgres -w -c "DROP DATABASE IF EXISTS {{DATABASE_NAME}};"

connect:
    psql {{DATABASE_NAME}}
