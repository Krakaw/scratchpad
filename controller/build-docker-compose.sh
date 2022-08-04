#!/bin/bash

TEMPLATE_FILE=$1
OUTPUT_FILE=$2
SERVICES_DIR="${3:-./docker-services.d/}"
VOLUMES_DIR="${4:-./docker-services.d/volumes/}"

TEMPLATE_STRINGS=""
for TEMPLATE in "${SERVICES_DIR}"*.yml; do
  TEMPLATE_STRING=$(tr <"$TEMPLATE" '\n' '§')
  TEMPLATE_STRINGS="$TEMPLATE_STRINGS§$TEMPLATE_STRING"
done

sed "s!{TEMPLATES}!${TEMPLATE_STRINGS//&/\\&}!" "$TEMPLATE_FILE" | tr '§' '\n' >"$OUTPUT_FILE"

if [[ -d "$VOLUMES_DIR" ]]; then
  VOLUMES_STRINGS=""
  for VOLUME in "${VOLUMES_DIR}"*.yml; do
    VOLUMES_STRING=$(tr <"$VOLUME" '\n' '§')
    VOLUMES_STRINGS="$VOLUMES_STRINGS§$VOLUMES_STRING"
  done

  sed "s!{VOLUMES}!${VOLUMES_STRING//&/\\&}!" "$OUTPUT_FILE" | tr '§' '\n' >"$OUTPUT_FILE.tmp"
  mv "$OUTPUT_FILE.tmp" "$OUTPUT_FILE"
fi
