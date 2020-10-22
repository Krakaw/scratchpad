#!/bin/bash

export DOCKER_BIN="${DOCKER_BIN:-$(which docker)}"
export CUID="${UID:-$(id -u)}"
export CGID="${GID:-$(id -g)}"
docker network create controller-network || echo "Network already exists"
if [[ -n "$REBUILD" ]]; then
  docker-compose build --no-cache controller
fi

if [[ -n "$DEV" ]]; then
  docker-compose up -f docker-compose.yml -f docker-compose.dev.yml up -d
else
  docker-compose up -d
fi

docker-compose exec controller sh -c "cron && cd /controller/controller && ./start-controller.sh"
