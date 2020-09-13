#!/bin/bash

OUTPUT_FILE=docker-compose.built.yml
cat << EOM > "$OUTPUT_FILE"
version: "3.4"
services:
EOM

for TEMPLATE in ./docker-services.d/*.yml; do
  cat "$TEMPLATE" >> "$OUTPUT_FILE"
done

cat << EOM >> "$OUTPUT_FILE"

networks:
  controller-network:
    external: true
EOM

