#!/bin/bash

if [ $# -ne 3 ]; then
    echo "usage: $0 <unix socket file> <host> <listen port>"
    exit
fi

SOCK=$1
HOST=$2
PORT=$3

[[ -e "${SOCK}" ]] && rm "${SOCK}"
socat -d UNIX-LISTEN:${SOCK},reuseaddr,fork TCP:${HOST}:${PORT} &
