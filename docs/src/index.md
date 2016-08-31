# ![Rote](logo.png)

Rote is a pleasantly simple task runner and build tool to supercharge your workflow.

## Why use a task runner?

Software has become much more complex since 1990, and it is only getting more complex. There is code to compile, test suites to run, containers to build, scripts to lint, and archives to deploy. Task runners help you to simplify your development workflow by making it simpler to automate some of these repetitive tasks. Put all of your tasks in one place and let a task runner to the boring work for you.

## Why use Rote?

Rote is designed to be a flexible, portable tool for any development environment. Just a single executable with no runtime dependencies, Rote can be installed on just about anything.

A task runner for all projects. Rote at its core concept is a simple task runner. Tasks are scripted in Lua, but nothing else about Rote is Lua-specific. The flexible, built-in scripting environment makes it easy to perform all kinds of complicated tasks with just a few lines of configuration.

A build tool for any language.

## Features

- Ridiculously fast.
- Task parallelization with threading.
- Incremental builds.
- Scriptable runtime.
- Familiar syntax that doesn't get in your way when you need to do some logic in your build.
- Completely portable binary with no system dependencies.

## Thanks

No software is an island. Both the ideas behind Rote and the technology it uses stand on the shoulders of the many who provide essential research, ideas, and code; often a thankless job. So here's our thanks to things that inspired Rote and projects that have helped Rote:

- [The Rust Programming Language](https://www.rust-lang.org), for an excellent native programming language;
- [Lua](https://www.lua.org), for such a simple, elegant scripting language;
- [rust-lua53](https://github.com/jcmoyer/rust-lua53), for Lua 5.3 bindings for Rust;
- [term](https://github.com/Stebalien/term), for simplifying cross-platform terminal colors;
- [GNU Make](https://www.gnu.org/software/make), for starting it all;
- [Jake](http://jakejs.com), for script syntax inspiration;
- [Cake](http://cakebuild.net), for script syntax inspiration;

## License

All documentation and source code is licensed under the Apache License, Version 2.0 (Apache-2.0). See the [LICENSE](LICENSE) file for details.
