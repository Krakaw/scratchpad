#!/bin/bash

export DOCKER_BIN="${DOCKER_BIN:-$(which docker)}"
export CUID="${UID:-$(id -u)}"
export CGID="${GID:-$(id -g)}"
docker network create controller-network || echo "Network already exists"
docker-compose up -d
docker-compose exec controller sh -c "cd /controller/controller && ./start-controller.sh"
