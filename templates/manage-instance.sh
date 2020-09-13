#!/bin/bash

export timestamp=${timestamp:-$(date +'%Y-%m-%d_%Hh%M%S')}
exec &>> >(tee -a "logs/$(basename $0)-$timestamp.txt") 2>> >(tee -a "logs/$(basename $0)-$timestamp.err")

source docker-source.sh

usage() {
  echo "$0 --update|--start|--stop|--restart|--wipe|--version [--web web_branch]"
}

generate_version() {
  VERSION=$(docker-compose run api-initialise api-cli version | grep -v "^{")
  echo "$VERSION" > api_version.txt
}

initialise() {
  docker-compose up api-initialise
}

start() {
  generate_version
  docker-compose up -d sockets
  docker-compose up -d logs
  docker-compose up -d bn-cube
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
  docker-compose up api-initialise
  docker-compose up -d api
}

web() {
  rm -rf "$BUILD_DIR/build"
  docker-compose pull web-build
  docker-compose up --no-deps web-build
  sed -i -E  "s/\"version\":\"(.*?)\"/\"version\":\"\1-$WEB_BRANCH\"/" "$BUILD_DIR/build/version.json"
  find "$BUILD_DIR/build/static/js/" -name 'main.*.js' -exec /usr/bin/perl -p -i -e  "s/window.bigneonVersion=\"(.*?)\"/window.bigneonVersion=\"\1-$WEB_BRANCH\"/g" {} +
}

update() {
  touch .
  generate_version
  docker-compose pull api
  docker-compose stop api
  docker-compose up -d sockets
  docker-compose up -d --no-deps api
}

get_env() {
  API_ENV=$(cat .api.env | grep -v "^\s*#" | sort | grep -v ^$)
  WEB_ENV=$(cat .web.env | grep -v "^\s*#" | sort | grep -v ^$)
  CUBE_ENV=$(cat .bn-cube.env | grep -v "^\s*#" | sort | grep -v ^$)
  echo -e "$API_ENV\n|--|\n$WEB_ENV\n|--|\n$CUBE_ENV"
}

reset_env_from_template() {
  FILE="$1"
  cp "../../templates/env.d/$FILE" ".$FILE"
  sed -i "s|__DB_NAME__|$DB_NAME|g" ".$FILE"
  sed -i "s|__API_BRANCH_URL__|$API_BRANCH_URL|g" ".$FILE"
}

if [ -z "$1" ]; then
  usage
  exit 1
fi

WEB_BRANCH=
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
    -v | --version)
      generate_version
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
