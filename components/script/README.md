# `runtime`
This crate provides the Lua interpreter that Rote scripts run in, and is used by all of the other crates.

`runtime` is essentially a high-level wrapper around [`rust-lua53`][rust-lua53], which is itself a wrapper around the [Lua] library. Most of the features provided are suitable for any Rust project using Lua, but there are also some methods that are specific to how Rote interacts with the Lua environment.


[lua]: https://www.lua.org
[rust-lua53]: https://github.com/jcmoyer/rust-lua53
