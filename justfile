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

create-network:
  docker network create shared_network

deploy:
  ./scripts/deploy.sh

undeploy:
  ./scripts/undeploy.sh

build-services:
  ./scripts/build-services.sh

deploy-services:
  ./scripts/deploy-services.sh

undeploy-services:
  ./scripts/undeploy-services.sh

