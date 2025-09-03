#!/bin/bash
set -e

echo "Building services..."
docker-compose --env-file .env -f services/docker-compose.yml build gateway
docker-compose --env-file .env -f services/docker-compose.yml build user
docker-compose --env-file .env -f services/docker-compose.yml build auth

echo "Building done!"
