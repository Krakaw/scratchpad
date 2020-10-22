#!/bin/bash

export DOCKER_BIN="${DOCKER_BIN:-$(which docker)}"
export CUID="${UID:-$(id -u)}"
export CGID="${GID:-$(id -g)}"
docker network create controller-network || echo "Network already exists"
if [[ -n "$REBUILD" ]]; then
  docker-compose build --no-cache controller
fi
docker-compose up -d
docker-compose exec controller sh -c "cron && cd /controller/controller && ./start-controller.sh"
