[working-directory: 'services']
generate-protos-rs:
  #!/usr/bin/env sh
  for d in */; do
    if [ -f $d/*.proto ]; then
      echo "Generating protos in $d";
      just -f $d/justfile generate-protos;
    fi
  done

[working-directory: 'app']
generate-protos-ts:
  just -f ./justfile generate-protos

generate-protos: generate-protos-rs generate-protos-ts

compose-build: 
  docker compose -f services/docker-compose.yml -f tracing/docker-compose.yml build

compose-up: 
  docker compose -f services/docker-compose.yml -f tracing/docker-compose.yml up

compose-down: 
  docker compose -f services/docker-compose.yml -f tracing/docker-compose.yml down -v

