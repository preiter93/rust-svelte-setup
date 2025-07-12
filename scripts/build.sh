#!/bin/bash
set -e

echo "Building services..."
docker-compose --env-file .env -f services/docker-compose.yml build

echo "Building done!"
