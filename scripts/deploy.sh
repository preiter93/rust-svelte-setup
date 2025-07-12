#!/bin/bash
set -e

echo "Starting DB..."
docker-compose --env-file .env -f infrastructure/db/docker-compose.yml up -d

echo "Waiting for DB to initialize..."
sleep 5

echo "Starting services..."
docker-compose --env-file .env -f services/docker-compose.yml up -d

echo "Starting Jaeger..."
docker-compose -f infrastructure/jaeger/docker-compose.yml up -d

echo "Deployment done!"
