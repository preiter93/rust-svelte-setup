#!/bin/bash
set -e

echo "Starting DB..."
docker-compose --env-file .env -f infrastructure/db/docker-compose.yml up -d

echo "Waiting 10 seconds for DB to initialize..."
sleep 10

echo "Starting services..."
docker-compose -f services/docker-compose.yml up -d

echo "Starting Jaeger..."
docker-compose -f infrastructure/jaeger/docker-compose.yml up -d

echo "Deployment done!"
