# Rote
Plain and simple task and build tool.

**This is highly experimental. Expect anything to change at any time.**

[![Build Status](https://img.shields.io/travis/coderstephen/rote.svg?style=flat-square)](https://travis-ci.org/coderstephen/rote)

Rote is an experimental task runner and build tool designed to be easy to use, portable, and fast. Rote uses [Lua](http://www.lua.org) as a configuration syntax. You do not need to be familiar with Lua to use Rote; the syntax is very simple to understand!

## Really? *Another* build tool?
The ecosystem for build and task tools is already so saturated that Rote can hardly offer anything new, right? Actually, that is the point of Rote entirely. Rote *intentionally* has less features than other tools like Ant or CMake, because too many features can make a build tool too complicated or slow.

The idea for Rote came from the failures of other existing tools. Here are just a few of the common issues:

- Nonstandard file formats or confusing configuration syntaxes.
- Making you write verbose code instead of defining tasks.
- Non-native tools themselves depend on interpreters to be installed, like Node.js, Python, or Ruby. This introduces additional complexity that does not come from a project itself, but with the build tool.

For more insulting comparisons to your favorite tool, here is a lengthier list:

- Make is lightweight and simple, but has a horrible, confusing syntax.
- Autotools are built on Make, but are the opposite of simple.
- Ant uses XML, which is incredibly verbose for even simple tasks. Plus XML isn't designed for logic.
- Phing is basically a PHP clone of Ant, so it inherits all the problems of Ant.
- Rake is written in Ruby.
- Grunt is a huge beast of a program.
- CMake? lol.

## Goals
- Completely portable binary with very few system dependencies.
- Familiar syntax that doesn't get in your way when you need to do some logic in your build.
- Build parallelization with threading.

## Compiling
Unfortunately, Rote can't build itself yet. For now, you can build it with Cargo:

    cargo build

This will compile Rote along with a downloaded Lua 5.3 interpreter.

*Note that Rote currently requires the nightly channel of the Rust compiler.

## Usage
To use Rote in your project, create a `Rotefile` in your project root. A `Rotefile` is a valid Lua script and should contain valid Lua code. Below is an example `Rotefile`:

```lua
function debug()
    cargo.build()
end

function release()
    cargo.build {
        release = true
    }
end

function clean()
    cargo.clean()
end

default = debug
```

Now to execute the `debug` task, we can run `rote debug`. Rote will look for the `Rotefile` in the current directory. If the file is in a different directory or has a different name, you can use the `-f` flag to specify a different location:

    rote -f my/Rotefile debug

See the `default = debug` at the end of the file? That sets the default task to `debug`. When `rote` is run without a task name, it assumes the "default" task should be run. To run the "debug" task then, we can just run

    rote

Tasks can also take arguments:

```lua
function echo(message)
    io.write(message, "\n")
end

default = debug
```

Running the `echo` task like this:

    rote echo "Hello, future!"

Will output:

    Hello, future!

See `rote -h` for more on command usage.

## License
All documentation and source code is licensed under the Apache License, Version 2.0 (Apache-2.0). See the [LICENSE](LICENSE) file for details.
