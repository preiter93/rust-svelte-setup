set dotenv-load := true

default:
    @just --list

# Build all backend services
[group: "build"]
build-services:
  docker compose --env-file .env -f services/docker-compose.yml build

# Builds a single backend service
[group: "build"]
build-service target:
  docker compose --env-file .env -f services/docker-compose.yml build {{ target }}

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

# Deploy the full system (DB, services, Jaeger, Traefik)
[group: "deploy"]
deploy:
  echo "Starting DB..."
  docker compose --env-file .env -f infrastructure/db/docker-compose.yml up -d

  echo "Waiting for DB to initialize..."
  sleep 5

  echo "Starting backend services..."
  docker compose --env-file .env -f services/docker-compose.yml up -d

  echo "Starting Traefik..."
  docker compose -f infrastructure/traefik/docker-compose.yml up -d

  echo "Starting Jaeger..."
  docker compose -f infrastructure/jaeger/docker-compose.yml up -d

  echo "Deployment complete!"

# Undeploys the full system (DB, services, Jaeger, Traefik)
[group: "deploy"]
undeploy:
  echo "Stopping Jaeger..."
  docker compose -f infrastructure/jaeger/docker-compose.yml down -v

  echo "Stopping Traefik..."
  docker compose -f infrastructure/traefik/docker-compose.yml down -v

  echo "Stopping services..."
  docker compose -f services/docker-compose.yml down

  echo "Stopping DB..."
  docker compose -f infrastructure/db/docker-compose.yml down -v

  echo "Undeployment complete!"

# Creates the docker network
[group: "deploy"]
create-network:
  docker network create shared_network

# Build the app
[group: "build"]
build-app:
  docker compose --env-file .env -f app/docker-compose.yml build

# Deploy the app
[group: "deploy"]
deploy-app:
  docker compose --env-file .env -f app/docker-compose.yml up -d

# Undeploy the app
[group: "deploy"]
undeploy-app:
  docker compose --env-file .env -f app/docker-compose.yml down

# Generate rust protobuf files
[working-directory: 'services']
[group: "generate"]
generate-protos-rs:
  #!/usr/bin/env sh
  set -e
  for d in */; do
    if [ -f "$d"/justfile ] && [ -n "$(find "$d" -name '*.proto' -print -quit)" ]; then
      echo "🧬 Generating protos in $d"
      just -f "$d"/justfile generate-protos
    fi
  done

# Generate rust protobuf files
[working-directory: 'services']
[group: "generate"]
generate-dockerfile:
  #!/usr/bin/env sh
  set -e
  for d in */; do
    if [ -f "$d"/justfile ] && [ "$d" != "pkg/" ]; then
      echo "🐳 Generating dockerfiles in $d"
      just -f "$d"/justfile generate-dockerfile
    fi
  done

# Generate typescript protobuf files
[working-directory: 'app']
[group: "generate"]
generate-protos-ts:
  @just -f ./justfile generate-protos

# Generate all protobuf files
[group: "generate"]
generate-protos: generate-protos-rs generate-protos-ts

# Generate protbuf files and dockerfiles
[group: "generate"]
generate: generate-protos-ts generate-protos-rs generate-dockerfile
