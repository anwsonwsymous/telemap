#### Available Filters

| **Filter Type** | **Example**                                       | **Description**                                                  | Feature   |
|-----------------|:--------------------------------------------------|------------------------------------------------------------------|-----------|
| **Incoming**    | -                                                 | _Attached by default to all pipelines, to prevent infinite loop_ | -         |
| **Text**        | `{"@type":"Text"}`                                | _Only text messages_                                             | -         |
| **Video**       | `{"@type":"Video"}`                               | _Only video messages_                                            | -         |
| **Photo**       | `{"@type":"Photo"}`                               | _Only photo messages_                                            | -         |
| **Animation**   | `{"@type":"Animation"}`                           | _Only animation messages_                                        | -         |
| **Document**    | `{"@type":"Document"}`                            | _Only document messages_                                         | -         |
| **AnyFile**     | `{"@type":"AnyFile"}`                             | _Any file content messages (video, photo, document, animation)_  | -         |
| **Counter**     | `{"@type":"Counter","count":5}`                   | _Every `nth` message_                                            | -         |
| **FileSize**    | `{"@type":"FileSize","size":2000,"op":"<"}`       | _Any file content's size in MB_                                  | -         |
| **Duration**    | `{"@type":"Duration","duration":60, "op":">"}`    | _Animation or video duration_                                    | -         |
| **TextLength**  | `{"@type":"TextLength","len":50,"op":">="}`       | _Text/caption length_                                            | -         |
| **Regexp**      | `{"@type":"Regexp","exp":"^[0-9]+$"}`             | _Messages which text/caption matches pattern_                    | -         |
| **WhiteList**   | `{"@type":"WhiteList","words":["hello","world"]}` | _This filter passes when message matches any of provided words_  | -         |
| **BlackList**   | `{"@type":"BlackList","words":["hello","world"]}` | _This filter rejects when message matches any of provided words_ | -         |
| **Unique**      | `{"@type":"Unique"}`                              | _Pass only unique messages_                                      | `storage` |
