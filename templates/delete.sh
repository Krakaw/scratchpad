#!/bin/bash

export timestamp=${timestamp:-$(date +'%Y-%m-%d_%H-%M-%S')}
exec &>> >(tee -a "logs/$(basename $0)-$timestamp.txt") 2>> >(tee -a "logs/$(basename $0)-$timestamp.err")

source docker-source.sh
source .api.env

BASE_PATH="$(cd "$(dirname "$0")" && pwd)"
DIR_NAME="$(basename "$BASE_PATH")"
echo "Deleting Scratch $DIR_NAME ..."

for DELETE_SCRIPT in ./scripts/delete.d/*.sh; do
  echo "Running delete script $DELETE_SCRIPT"
  "$DELETE_SCRIPT"
done

echo "Removing docker images"
docker-compose down --rmi local --remove-orphans

echo "Deleting base dir "
cd ..
rm -rf "$BASE_PATH"
