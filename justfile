set dotenv-load := true

default:
    @just --list

# Build all backend services
[group: "build"]
build-services:
  docker compose --env-file .env -f services/docker-compose.yml build

# Build backend services synchronously
[group: "build"]
build-services-sync:
  docker compose --env-file .env -f services/docker-compose.yml build gateway
  docker compose --env-file .env -f services/docker-compose.yml build user
  docker compose --env-file .env -f services/docker-compose.yml build auth

# Deploy all services
[group: "deploy"]
deploy-services:
  docker compose --env-file .env -f services/docker-compose.yml up -d

# Undeploy all services
[group: "deploy"]
undeploy-services:
  docker compose --env-file .env -f services/docker-compose.yml down

# Deploy the full system (DB, services, Jaeger)
[group: "deploy"]
deploy:
  echo "Starting DB..."
  docker compose --env-file .env -f infrastructure/db/docker-compose.yml up -d

  echo "Waiting for DB to initialize..."
  sleep 5

  echo "Starting backend services..."
  docker compose --env-file .env -f services/docker-compose.yml up -d

  echo "Starting Jaeger..."
  docker compose -f infrastructure/jaeger/docker-compose.yml up -d

  echo "Deployment complete!"

# Undeploy everything — stops Jaeger, services, and DB
[group: "deploy"]
undeploy:
  echo "Stopping Jaeger..."
  docker compose -f infrastructure/jaeger/docker-compose.yml down -v

  echo "Stopping services..."
  docker compose -f services/docker-compose.yml down

  echo "Stopping DB..."
  docker compose -f infrastructure/db/docker-compose.yml down -v

  echo "Undeployment complete!"

# Creates the docker network
[group: "deploy"]
create-network:
  docker network create shared_network

# Generate rust protobuf files
[working-directory: 'services']
[group: "protos"]
generate-protos-rs:
  #!/usr/bin/env sh
  set -e
  for d in */; do
    if [ -f "$d"/justfile ] && [ -n "$(find "$d" -name '*.proto' -print -quit)" ]; then
      echo "🧬 Generating protos in $d"
      just -f "$d"/justfile generate-protos
    fi
  done


# Generate typescript protobuf files
[working-directory: 'app']
[group: "protos"]
generate-protos-ts:
  just -f ./justfile generate-protos

# Generate all protobuf files
[group: "protos"]
generate-protos: generate-protos-rs generate-protos-ts
