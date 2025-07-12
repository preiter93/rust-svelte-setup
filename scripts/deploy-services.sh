#!/bin/bash
set -e

docker-compose --env-file .env -f services/docker-compose.yml up
