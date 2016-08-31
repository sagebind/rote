# Extending Rote

Rote provides many useful features for writing tasks and build scripts built-in, but by no means does it have a convenient function for every use case! One of Rote's core values is extensibility to allow script authors to tailor Rote for the types of tasks they need to perform.

Extending Rote is as simple as writing a standard Lua module. The standard `require` function is available automatically and works on both Lua and native modules.

For more details on Lua modules and the `require` function, see [section 6.3 of the Lua manual][lua-manual-6.3].

## Creating a custom module

The easiest and most portable way of creating a custom module is to follow the normal Lua conventions for modules. Let's start out with a quick example:

```lua
local clean = {}

-- Cleans a path of unwanted files
function clean.clean(path, force, noWrite)
    -- todo: Implement cleaning
end

return clean
```

This defines a module called `clean` that provides a single function called `clean()`. To make this a module, we can put the above code into a file called `clean.lua`.

## Reserved names

While you can name Lua modules almost anything, Rote reserves a few module names for the modules that it provides built-in.


[lua-manual-6.3]: https://www.lua.org/manual/5.3/manual.html#6.3
