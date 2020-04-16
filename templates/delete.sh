#!/bin/bash

export timestamp=${timestamp:-$(date +'%Y-%m-%d_%Hh%M%S')}
exec &>> >(tee -a "logs/$(basename $0)-$timestamp.txt") 2>> >(tee -a "logs/$(basename $0)-$timestamp.err")

source docker-source.sh
source .api.env

BASE_PATH="$(cd "$(dirname "$0")" && pwd)"
DIR_NAME="$(basename "$BASE_PATH")"
echo "Deleting Scratch $DIR_NAME ..."

echo "Dropping database"
docker-compose stop api
docker-compose run api bndb_cli drop -c ${DATABASE_URL}
echo "Removing docker images"
docker-compose down --rmi local --remove-orphans

echo "Deleting base dir "
cd ..
rm -rf "$BASE_PATH"
