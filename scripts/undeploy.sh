#!/bin/bash
set -e

echo "Stopping Jaeger..."
docker-compose -f infrastructure/jaeger/docker-compose.yml down

echo "Stopping services..."
docker-compose -f services/docker-compose.yml down

echo "Stopping DB..."
docker-compose -f infrastructure/db/docker-compose.yml down

echo "Undeployment complete!"
