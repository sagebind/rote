-- Core functions provided by the Rote environment. This module is priveleged
-- and is always loaded before parsing any configuration files.

rote = {}

local print_raw = print

-- Expands global and environment variables inside a given string.
function rote.expand(str)
    return string.gsub(str, "$(%w+)", function (name)
        return os.getenv(name) or _G[name] or ""
    end)
end

-- Escapes a string allowing it to be passed safely to a shell function.
function rote.escape_arg(str)
    return "'" .. string.gsub(str, "'", "\\'") .. "'"
end

-- Executes a shell command with a given list of arguments.
function rote.execute(cmd, ...)
    for i,arg in ipairs({...}) do
        cmd = cmd .. " " .. rote.escape_arg(arg)
    end

    rote.print(rote.expand(cmd));
    return os.execute(rote.expand(cmd))
end

-- Prints a string to standard output.
function rote.print(str)
    str = str or ""
    print_raw(rote.expand(str))
end

-- Parses an input table of options and merges it with a table of default values.
function rote.options(given, defaults)
    if given ~= nil then
        for k,v in pairs(given) do
            defaults[k] = v
        end
    end

    return defaults
end

-- Define some global function aliases.
exec = rote.execute
print = rote.print

return rote
