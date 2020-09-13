#!/bin/bash

./build-docker-compose.sh docker-compose.template.yml docker-compose.built.yml
docker-compose -f docker-compose.built.yml up -d
