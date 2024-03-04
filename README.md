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
            --volume $(pwd)/config:/config \
            --volume $(pwd)/.env:/app/.env \
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

How it works
===

Program accepts `json` config path as argument, which consists of 2 parts: Maps and Pipelines.

### Maps

This is `array` of map objects. Map object has `src` and `dest` fields.
`src` is incoming message's `chat_id` and is required. `dest` is array of destination `chat_ids` where received message will be sent/mapped.

```json
{
  "maps": [
    {"src": 1, "dest": [2, 3, 4]}
  ]
}
```
> _Explain: All incoming messages in chat with id `1` will be sent/mapped to chats with id `2`,`3` and `4`._

`dest` is optional. If it's not provided, it'll be set automatically the same as `src` is.

`{"src": 1}` is the same as `{"src": 1, "dest": [1]}`

### Pipelines

This is `array` of pipeline objects. Pipeline object describes how the content received in one chat must be mapped to another chat, what filters and pipes must be applied.

```json
{
 "pipelines": [
   {
     "name": "Map only video messages.",
     "route": {"src": 1, "dest": 2},
     "filters": [{"@type": "Video"}]
   }
 ] 
}
```
> _Explain: Only `video` messages received in the chat with id `1` will be sent/mapped to the chat with id `2`._

#### Routing

`route` field describes the path from source chat to destination chat. (`src` and `dest`)
Both source and destination fields are optional. So if no route provided it would be automatically set to `{"src": 0, "dest": 0}` which is so-called default pipeline.
While trying to find pipelines to process received message, the highest priority have routes with both `src` and `dest` provided, then with only `dest` provided and finally routes with `src` only.
If mapping exists for route but pipeline not found then default pipeline will be used.

```json
 {
  "maps": [
    {"src": 1, "dest": [2, 3, 4, 5]},
    {"src": 6, "dest": [7]}
  ],
  "pipelines": [
    {
      "name": "FIRST",
      "route": {"src": 1, "dest": 2}
    },
    {
      "name": "SECOND",
      "route": {"dest": 3}
    },
    {
      "name": "THIRD",
      "route": {"src": 1}
    }
  ]
}
```
> _Explain: If we receive message in chat `1`, "FIRST" pipeline will be used to send it to chat `2`, "SECOND" pipeline will be used to send to chat `3`, "THIRD" pipeline for chat `4` and `5`. If message received in chat `6`, default pipeline will be used._

By default, the "default" pipeline acts like `echo` server.

```json
{
  "filters": [{"@type": "Incoming"}],
  "pipes": [{"@type": "Transform"}]
} 
```
> _Explain: Default pipeline._

There could be multiple pipelines with the same routing.

```json 
{
  "pipelines": [
    {
      "name": "Map only video messages.",
      "route": {"src": 1, "dest": 2},
      "filters": [{"@type": "Video"}],
      "pipes":[{"@type":"StaticText","formatted_text":{"text":"Video Message"}}]
    },
    {
      "name": "Map only photo messages.",
      "route": {"src": 1, "dest": 2},
      "filters": [{"@type": "Photo"}],
      "pipes":[{"@type":"StaticText","formatted_text":{"text":"Photo Message"}}]
    }
  ]
}
```
> _Explain: Multiple pipelines for the same route, Video messages received in the chat 1, send/map to the chat 2 with caption "Video Message" and Photo messages received in the chat 1, send/map to the chat 2 with caption "Photo Message"._


#### Filters

| **Filter Type** | **Example**                                    | **Description**                                                  | Feature   |
|-----------------|:-----------------------------------------------|------------------------------------------------------------------|-----------|
| **Incoming**    | -                                              | _Attached by default to all pipelines, to prevent infinite loop_ | -         |
| **Video**       | `{"@type":"Video"}`                            | _Only video messages_                                            | -         |
| **Photo**       | `{"@type":"Photo"}`                            | _Only photo messages_                                            | -         |
| **Animation**   | `{"@type":"Animation"}`                        | _Only animation messages_                                        | -         |
| **Document**    | `{"@type":"Document"}`                         | _Only document messages_                                         | -         |
| **File**        | `{"@type":"File"}`                             | _Any file content messages (video, photo, document, animation)_  | -         |
| **Counter**     | `{"@type":"Counter","count":5}`                | _Every `nth` message_                                            | -         |
| **FileSize**    | `{"@type":"FileSize","size":2000,"op":"<"}`    | _Any file content's size in MB_                                  | -         |
| **Duration**    | `{"@type":"Duration","duration":60, "op":">"}` | _Animation or video duration_                                    | -         |
| **TextLength**  | `{"@type":"TextLength","len":50,"op":">="}`    | _Text/caption length_                                            | -         |
| **RegExp**      | `{"@type":"RegExp","exp":"^[0-9]+$"}`          | _Messages which text/caption matches pattern_                    | -         |
| **Unique**      | `{"@type":"Unique"}`                           | _Pass only unique messages_                                      | `storage` |

#### Pipes

| **Pipe Type**     | **Example**                                                                   | **Description**                                                                                                                                        |
|-------------------|:------------------------------------------------------------------------------|--------------------------------------------------------------------------------------------------------------------------------------------------------|
| **Transform**     | -                                                                             | _Attached by default to all pipelines.Transforms Input message into Output message as it is_                                                           |
| **StaticText**    | `{"@type":"StaticText","formatted_text":{"text":"Hola"}}`                     | _Set text/caption on output message_                                                                                                                   |
| **Replace**       | `{"@type":"Replace","search": ["text1", "text2"],"replace": "replaced text"}` | _Search and replace text on output message_                                                                                                            |
| **ReplaceRegExp** | `{"@type":"ReplaceRegExp","search": ".","replace": "*", "all": true}`         | _Search and replace texts with regular expression. By default all occurrences should be replaced. Use option `"all": false` for replacing only first._ |


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
          anwsonwsymous/telemap:slim
# Run with cargo
cargo run -- -c /app/config/test.json
# Run tests
cargo  test
```
