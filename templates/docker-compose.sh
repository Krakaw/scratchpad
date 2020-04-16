#!/bin/bash

export timestamp=${timestamp:-$(date +'%Y-%m-%d_%Hh%M%S')}
exec &>> >(tee -a "logs/$(basename $0)-$timestamp.txt") 2>> >(tee -a "logs/$(basename $0)-$timestamp.err")

source ./docker-source.sh

docker-compose $@
