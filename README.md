<h1 align="left">
  Telemap
</h1>

<p align="left">
  <a href="https://github.com/anwsonwsymous/telemap/blob/master/LICENSE">
    <img src="https://img.shields.io/static/v1?label=License&message=MIT&color=blue" alt="Telegram account">
  </a>
  <a href="https://github.com/anwsonwsymous/telemap/actions/workflows/test.yml">
    <img src="https://github.com/anwsonwsymous/telemap/actions/workflows/test.yml/badge.svg" alt="Tests status"/>
  </a>
</p>
<p><strong>The goal of this app is to send/map received messages in one chat to another chat applying filters and mutations to them.</strong></p>

[See here how it works, with examples](EXPLAIN.md)

Usage
===

Currently, there are 2 ways to run this program.

* [Build **docker** image and run container](#run-in-docker)
* [Build **tdlib** and link with **cargo**](#run-manually)

Steps below are required for both options

- Copy `.env.example` with name `.env` and fill your telegram credentials.
  > `TELEGRAM_API_ID`,`TELEGRAM_API_HASH` and `TELEGRAM_PHONE` are required.
  >
  > Take creds from https://my.telegram.org/apps
- Create `json` configuration file. (examples: `config/answer_machine.json.example`, `config/echo.json`)


## Run in docker

- Build image `docker build -t anwsonwsymous/telemap:slim .`
  > On `arm64` architecture use `--platform` option to emulate `amd64` 
  > 
  > `docker build --platform linux/amd64 -t anwsonwsymous/telemap:slim .`
  > 

- Copy `config/echo.json.example` to `config/echo.json` and put your chat `id`
- Run container and pass newly created `json` config path
  ```shell
  docker run \
            --rm \
            --init \
            --interactive \
            --tty \
            --memory 25m \
            --volume $(pwd)/telegram_database:/app/telegram_database \
            --volume $(pwd)/storage:/app/storage \
            --volume $(pwd)/config:/config \
            --volume $(pwd)/.env:/app/.env \
            --env-file $(pwd)/.env
            --name telemap-app \
            anwsonwsymous/telemap:slim -c /config/echo.json
  ```

## Run manually

Build `tdjson` [manually](https://core.telegram.org/tdlib/docs/#building) or [copy from docker image](#copy_tdjson_from_docker). 

For linking `tdjson` either use `RUSTFLAGS` or `LD_LIBRARY_PATH` environment variable.

Currently, supported version is `18.0.0`

Run with `cargo`

```shell
RUSTFLAGS="-C link-args=-Wl,-rpath,/tdjson/path,-L/tdjson/path" cargo run -- -c /app/config/answer_machine.json
```


<a href="copy_tdjson_from_docker"></a>
#### Copy tdjson from docker image

```shell
id=$(docker create wcsiu/tdlib:1.8.0)
docker cp $id:/td/build/libtdjson.so.1.8.0 /tdjson/path
docker cp -L $id:/td/build/libtdjson.so /tdjson/path
docker rm -v $id
```


## Processing Components

Find more information on pipes and filters in their respective directories:

[List of available filters](src/processing/filters/README.md)

[List of available pipes](src/processing/pipes/README.md)


## Contributing

Pull requests are welcome. Please open an issue first to discuss what you would like to change.

Please make sure to update tests and README as appropriate.

### For Development

- Run container with project directory `volume`
- Make code changes 
- Run with cargo 

```shell
# Run container with project directory volume
docker run \
          --interactive \
          --tty \
          --name telemap-dev \
          --entrypoint /bin/bash \
          --volume $(pwd):/app \
          --env-file $(pwd)/.env
          anwsonwsymous/telemap:slim
# Run with cargo
cargo run -- -c /app/config/test.json
# Run tests
cargo  test
```
