# Getting started

## Installing

Rote supports nearly any platform that Rust supports.

The Rote command-line tool is a self-contained binary program that has very minimal system dependencies, so Rote can be run just about anywhere.

```sh
$ git clone https://github.com/sagebind/rote
$ cd rote
$ ./install.sh
```

### Compiling it yourself

If you are developing Rote or need to run it on an unsupported operating system, you can compile the Rote executable yourself. The CLI is written in [Rust], so you will need to set up a Rust compilation environment first. You will also need Cargo, the Rust dependency manager. See the [Rust documentation](https://doc.rust-lang.org/stable/book/installing-rust.html) for more on installing Rust.

Once you have `rustc` and `cargo` installed, you can compile Rote with

```sh
$ cargo build --release
```

## Basic usage

Using Rote is straightforward: we describe how to perform some [tasks](tasks.md), and then Rote executes them. Tasks are defined as functions using the [Lua] scripting language, and placed into a [Rotefile](rotefiles.md). To run one or more tasks that you have defined, you invoke the `rote` command-line utility, which parses the script file and runs the requested task(s).



[lua]: https://www.lua.org
[rust]: https://www.rust-lang.org
