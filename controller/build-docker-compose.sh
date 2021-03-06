#!/bin/bash

TEMPLATE_FILE=$1
OUTPUT_FILE=$2
SERVICES_DIR="${3:-./docker-services.d/}"

TEMPLATE_STRINGS=""
for TEMPLATE in "${SERVICES_DIR}"*.yml; do
  TEMPLATE_STRING=$(tr <"$TEMPLATE" '\n' '§')
  TEMPLATE_STRINGS="$TEMPLATE_STRINGS§$TEMPLATE_STRING"
done

sed "s!{TEMPLATES}!${TEMPLATE_STRINGS//&/\\&}!" "$TEMPLATE_FILE" | tr '§' '\n' >"$OUTPUT_FILE"
