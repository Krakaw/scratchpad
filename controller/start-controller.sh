#!/bin/bash

for SCRIPT in ./scripts/initialise.d/*.sh; do
  "$SCRIPT"
done
./build-docker-compose.sh docker-compose.template.yml docker-compose.built.yml
docker-compose -f docker-compose.built.yml up -d
