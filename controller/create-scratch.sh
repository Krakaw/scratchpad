#!/bin/bash
set -e

export timestamp=${timestamp:-$(date +'%Y-%m-%d_%H-%M-%S')}
exec &>> >(tee -a "logs/$(basename $0)-$timestamp.txt") 2>> >(tee -a "logs/$(basename $0)-$timestamp.err")

usage() {
  echo "API Branch required"
  echo "$0 --api api_branch_name [--name release_name_or_alias] [--web web_branch_name] [--db-prefix db_prefix]"
}

if [[ $# -eq 0 ]]; then
  usage
  exit 1
fi

DB_PREFIX="scratchpad_"
BASE_PATH="$(cd "$(dirname "$0")" && cd ../ && pwd)"
BASE_RELEASE_PATH="${BASE_PATH}/releases"
TEMPLATES_DIR="${BASE_PATH}/templates"
CONTROLLER_DIR="${BASE_PATH}/controller"
API_BRANCH=
while [ "$1" != "" ]; do
  case $1 in
  -a | --api)
    shift
    API_BRANCH=$1
    ;;
  -n | --name)
    shift
    RELEASE_NAME=$1
    ;;
  -w | --web)
    shift
    WEB_BRANCH=$1
    ;;
  -b | --build)
    shift
    BUILD_DIR=$1
    ;;
  -d | --db-prefix)
    shift
    DB_PREFIX=$1
    ;;
  -h | --help)
    usage
    exit
    ;;
  *)
    usage
    exit 1
    ;;
  esac
  shift
done

if [[ -z $API_BRANCH ]]; then
  usage
  exit 1
fi

RELEASE_NAME="${RELEASE_NAME:-$API_BRANCH}"
API_BRANCH_URL=${RELEASE_NAME//[^[:alnum:]-_]/}
WEB_BRANCH=${WEB_BRANCH:-primary}
DB_NAME="${DB_PREFIX}${API_BRANCH_URL}"
RELEASE_PATH="${BASE_RELEASE_PATH}/$API_BRANCH_URL"
BUILD_DIR="${RELEASE_PATH}/${BUILD_DIR:-web}"

# The instance already exists, just pull the latest
if [ -d "$BUILD_DIR" ]; then
  cd "$RELEASE_PATH" && ./manage-instance.sh --update
  exit 0
fi

# Create required folders
REQUIRED_PATHS=("$BUILD_DIR" "$RELEASE_PATH/logs" "$RELEASE_PATH/socks" "$RELEASE_PATH/storage" "$RELEASE_PATH/env.d")
for REQUIRED_PATH in "${REQUIRED_PATHS[@]}"; do
  mkdir -p "$REQUIRED_PATH" || exit 1
done

# Create required files
TOUCH_FILES=("$RELEASE_PATH/logs/api.log" "$RELEASE_PATH/logs/web.log")
for TOUCH_FILE in "${TOUCH_FILES[@]}"; do
  touch "$TOUCH_FILE" || exit 1
done

# Set permissions for release dir
chown -R "${CUID}:${CGID}" "$RELEASE_PATH"
chmod -R g+s "$RELEASE_PATH"

############################## Now we move into the release path
cd "${RELEASE_PATH}" || exit 1

LINK_FILES=("docker-compose.template.yml" "manage-instance.sh" "docker-compose.sh" "delete.sh" "scripts" "docker-services.d")
for LINK_FILE in "${LINK_FILES[@]}"; do
  ln -sr "../../templates/$LINK_FILE" "./"
done

# Specifically link the build-docker-compose.sh file
ln -sr "../../controller/build-docker-compose.sh" "./"

#Manually link the template fallback index.html
ln -sr "../../templates/building.html" "$BUILD_DIR/"
# Build the source file
cat <<EOM >docker-source.sh
export HOST_RELEASE_PATH=$HOST_RELEASE_PATH
export RELEASE_NAME=$RELEASE_NAME
export BASE_RELEASE_PATH=$BASE_RELEASE_PATH
export RELEASE_PATH=$RELEASE_PATH
export BUILD_DIR=$BUILD_DIR
export API_BRANCH=$API_BRANCH
export API_BRANCH_URL=$API_BRANCH_URL
export WEB_BRANCH=$WEB_BRANCH
export DB_NAME=$DB_NAME
export CUID=$CUID
export CGID=$CGID
EOM
source docker-source.sh

shopt -s nullglob
for GENERATE_ENV in ../../templates/env.d/.[^.]*.env; do
  echo "Generate $GENERATE_ENV"
  ./manage-instance.sh --reset-env "${GENERATE_ENV##*/}"
done

echo "Initialising"
./manage-instance.sh --initialise

# Build the web
echo "Building the web ..."
./manage-instance.sh --web "$WEB_BRANCH"

echo "Starting"
./manage-instance.sh --start
echo "Done."
