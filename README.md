# ![Rote](/docs/logo.png?raw=true)
##### Automate everything.

[![Build Status](https://img.shields.io/travis/coderstephen/rote.svg)](https://travis-ci.org/coderstephen/rote) [![Release](https://img.shields.io/github/release/coderstephen/rote.svg)]() [![Downloads](https://img.shields.io/github/downloads/coderstephen/rote/total.svg)]()

Rote is a multi-threaded task runner and build tool designed to be easy to use, portable, and fast. Automate your entire development workflow using Rote's scriptable task system to implement builds, deployment, and maintenance using a unified script syntax.


## Features
- Ridiculously fast.
- Task parallelization with threading.
- Incremental builds.
- Scriptable runtime.
- Familiar syntax that doesn't get in your way when you need to do some logic in your build.
- Completely portable binary with no system dependencies.


## Compiling
Rote comes with some shell scripts to orchestrate building. If only there was a good build tool we could use instead...

    ./scripts/build.sh

This will compile Rote along with a downloaded Lua 5.3 interpreter.


## Usage
To use Rote in your project, create a `Rotefile` in your project root. A `Rotefile` is a valid Lua script and should contain valid Lua code. Below is an example `Rotefile`:

```lua
require "cargo"
default "debug"

task("debug", function()
    cargo.build()
end)

task("release", function()
    cargo.build {
        release = true
    }
end)

task("clean", function()
    cargo.clean()
end)
```

Rote uses [Lua](http://www.lua.org) as a configuration syntax. You do not need to be familiar with Lua to use Rote; the syntax is very simple to understand!

Now to execute the `debug` task, we can run `rote debug`. Rote will look for the `Rotefile` in the current directory. If the file is in a different directory or has a different name, you can use the `-f` flag to specify a different location:

    rote -f my/Rotefile debug

See the `default "debug"` near the top? That sets the default task to `debug`. When `rote` is run without a task name, it assumes the "default" task should be run. To run the "debug" task then, we can just run

    rote

See `rote -h` for more on command usage.


## Batteries included
Since your task runner and build tool typically runs before your dependency managers, it makes little sense for you to have to install a plethora of plugins before running tasks. That's why Rote includes many common tasks built-in directly; Rote comes batteries included.

If there is a reusable component you'd like to use, but keep out of your actual `Rotefile`, you can save it as a simple Lua module too inside your project repository, or in one of your system's Lua include paths. Then using it in your `Rotefile` is as simple as requiring the module by name:

```lua
require "my_custom_module"
```


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


## Acknowledgements
Like most amazing software, Rote stands on the shoulders of the many who provide essential libraries and components, often a thankless job. So here's our thanks to some projects we'd like to highlight that have inspired Rote or are being used in Rote:

- [The Rust Programming Language](https://www.rust-lang.org), for an excellent native programming language
- [Lua](https://www.lua.org), for such a simple, elegant scripting language
- [rust-lua53](https://github.com/jcmoyer/rust-lua53), for Lua 5.3 bindings for Rust
- [term](https://github.com/Stebalien/term), for simplifying cross-platform terminal colors
- [GNU Make](https://www.gnu.org/software/make), for starting it all
- [Jake](http://jakejs.com), for script syntax inspiration
- [Cake](http://cakebuild.net), for script syntax inspiration


## License
All documentation and source code is licensed under the Apache License, Version 2.0 (Apache-2.0). See the [LICENSE](LICENSE) file for details.
