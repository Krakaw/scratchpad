#!/bin/bash

CMD=$@
find ${RELEASE_BASE}/releases/ -mindepth 1 -maxdepth 1 -type d -exec bash -c 'i="$1"; cd $1 && $2' _ {} "${CMD}" \;
