<h1 align="center">Welcome to scratchpad 👋</h1>
<p>
  <img alt="Version" src="https://img.shields.io/badge/version-0.1.0-blue.svg?cacheSeconds=2592000" />
</p>

> Deploy &#34;scratch&#34; environments easily

### ✨ [Scratchpad](https://github.com/Krakaw/scratchpad)

## Install

```sh
git clone git@github.com:Krakaw/scratchpad.git
```

## Usage

```sh
cp .controller.env.sample .controller.env
cp controller/.mockchain.env.sample controller/.mockchain.env
cp controller/.pg.env.sample controller/.pg.env
./start.sh
```

## What's Happening

1. `start.sh` will start the controller docker container that has a node.js web server, it also starts an `inotifywait` command to monitor the host nginx configs.
2. Using `network_mode: host` it listens directly the the `PORT` specified. This also gives access to `netstat` which allows the container to find open ports to start new scratches on.
3. The `controller` docker has access to `docker.sock` so it can spin up the main environment (pg, redis, mockchain) as well as turn on and off the scratches.
4. Each scratch gets its own copy of the `templates/.*.env` and symlinked the other control files.
5. The main complexity is in the nginx reload until we move to a dockerized nginx

## Unix Sockets

1. Adding a new docker image that binds a volume and creates 3 socket files
2. The socat image in the docker-compose.yml can use the socat-to-scp.sh script to forward traffic to the container
3. The host nginx can use the uri either `*`.example.com or example.com/`*` depending on the ssl setup to know which socket to proxy to
