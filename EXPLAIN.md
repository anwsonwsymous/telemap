How It Works
==

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
