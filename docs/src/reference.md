Complete reference to all built-in functions and modules provided by Rote for scripts.

## rote

This module provides the core set functions available in every build script. The `rote` module is always implicitly `require`d.

### `rote.create_task(name, deps={}, action=nil)`

Defines a new task with the name given by `name`.

### `rote.create_rule()`
### `rote.change_dir()`
### `rote.current_dir()`
### `rote.current_exe()`
### `rote.env()`
### `rote.execute()`
### `rote.expand()`
### `rote.export()`
### `rote.glob()`
### `rote.merge()`
### `rote.pipe()`
### `rote.print()`
### `rote.set_default_task()`
### `rote.version()`


## fs
### `fs.exists()`
### `fs.is_dir()`
### `fs.is_file()`
### `fs.is_symlink()`
### `fs.mkdir()`
### `fs.copy()`
### `fs.rename()`
### `fs.remove()`
### `fs.get()`
### `fs.put()`
### `fs.append()`
### `fs.combine()`


## http
### `http.get()`
### `http.post()`


## json
### `json.parse(json)`
Parses a JSON string into appropriate native values and returns the result.

### `json.stringify(value, pretty=false, spaces=4)`
Converts `value` into an appropriate JSON string representation. If `pretty` is set to `true`, the string is formatted for maximum readability instead of storage efficiency, using `spaces` number of spaces as an indentation amount.


## cpp
### `cpp.binary()`


## java
### `java.binary()`
