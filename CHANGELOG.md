CHANGELOG
===
## v0.6.0
* new pipe - `Format`
## v0.5.0
* new pipe - `StaticPhoto`
## v0.4.0
* new filter - `BlackList` words
* new filter - `WhiteList` words
* new filter - `Text` messages only
* rename filter `File` to `AnyFile`
* message type filters refactoring (`Video`,`Photo`,`Animation`,`Document`,`File`)
* code structure refactoring. pipes and filters in different directories.
## v0.3.0
* new pipe - `ReplaceRegexp`
* update pipe - `Replace` now receives array of strings as `search` parameter for multiple replacements
## v0.2.0
* new pipe - `Replace`
* new filter - `Unique`
* new feature `Storage` with dependencies `pickledb` and `md5`
## v0.1.0
* Initial version
