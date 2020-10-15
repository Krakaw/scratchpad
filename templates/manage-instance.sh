#!/bin/bash

export timestamp=${timestamp:-$(date +'%Y-%m-%d_%H-%M-%S')}
exec &>> >(tee -a "logs/$(basename $0)-$timestamp.txt") 2>> >(tee -a "logs/$(basename $0)-$timestamp.err")

source docker-source.sh

usage() {
  echo "$0 --update|--start|--stop|--restart|--wipe|--version [--web web_branch]"
}

initialise() {
  # Generate the docker-compose.yml file
  ./build-docker-compose.sh docker-compose.template.yml docker-compose.yml ./docker-services.d/
  shopt -s nullglob
  for SCRIPT in ./scripts/initialise.d/*.sh; do
    echo "Running script $SCRIPT"
    bash "$SCRIPT" || echo "Error in $SCRIPT"
  done
}

start() {
  docker-compose up -d sockets
  docker-compose up -d logs
  docker-compose up -d web
  docker-compose up -d --no-deps api
}

stop() {
  docker-compose down
}

restart() {
  stop
  start
}

rebuild() {
  restart
  web
}

wipe_db() {
  docker-compose stop api
  [ -f ./scripts/initialise.d/api.sh ] && bash ./scripts/initialise.d/api.sh
  docker-compose up -d api
}

web() {
  rm -rf "$BUILD_DIR/build/*"
  docker-compose pull web
  docker-compose up --no-deps web
  [ -f ./scripts/up.d/web.sh ] && bash ./scripts/up.d/web.sh

}

update() {
  touch .
  docker-compose pull
  docker-compose stop api
  docker-compose up -d sockets
  docker-compose up -d --no-deps api
}

get_env() {
  OUTPUT=""
  shopt -s nullglob
  for ENV_FILE in env.d/.[^.]*.env; do
      FILE_CONTENTS=$(grep -v "^\s*#" "$ENV_FILE" | sort | grep -v ^$)
      OUTPUT="$OUTPUT|--|$ENV_FILE|--|\n$FILE_CONTENTS\n"
  done
  echo -e "$OUTPUT"
}

reset_env_from_template() {
  TEMPLATE_FILE="$1"
  LOCAL_FILE="env.d/${TEMPLATE_FILE##*/}"
  cp "$TEMPLATE_FILE" "$LOCAL_FILE"
  sed -i "s|__DB_NAME__|$DB_NAME|g" "$LOCAL_FILE"
  sed -i "s|__API_BRANCH_URL__|$API_BRANCH_URL|g" "$LOCAL_FILE"
}

if [ -z "$1" ]; then
  usage
  exit 1
fi

WEB_BRANCH=primary
while [ "$1" != "" ]; do
  case $1 in
    -i | --initialise)
      initialise
      ;;
    -u | --update)
      update
      ;;
    -s | --start)
      start
      ;;
    -S | --stop)
      stop
      ;;
    -r | --restart)
      restart
      ;;
    -R | --rebuild)
      rebuild
      ;;
    -W | --wipe)
      wipe_db
      ;;
    -w | --web)
      shift
      export WEB_BRANCH=$1
      web
      ;;
    -e | --env)
      get_env
      ;;
    -t | --reset-env)
      shift
      reset_env_from_template "$1"
      ;;
    -h | --help)
      usage
      ;;
    *)
      usage
      exit 1
      ;;
  esac
  shift
done
