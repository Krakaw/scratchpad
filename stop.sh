#!/bin/bash

docker-compose exec controller bash -c "cd /controller/controller && ./for-each-release.sh ./manage-instance.sh --stop"
docker-compose exec controller sh -c "cd /controller/controller && docker-compose down"
docker-compose down
