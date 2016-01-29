-- Core functions provided by the core environment. This module is priveleged
-- and is always loaded before parsing any configuration files.
core = {}

local print_raw = print
local export_raw = export


-- Expands global and environment variables inside a given string.
function core.expand(str)
    return string.gsub(str, "$(%w+)", function (name)
        return os.getenv(name) or _G[name] or ""
    end)
end

-- Exports an environment variable.
function core.export(key, value)
    return export_raw(key, core.expand(value))
end

-- Escapes a string allowing it to be passed safely to a shell function.
function core.escape_arg(str)
    return "'" .. string.gsub(str, "'", "\\'") .. "'"
end

-- Executes a shell command with a given list of arguments.
function core.execute(cmd, ...)
    for i,arg in ipairs({...}) do
        cmd = cmd .. " " .. core.escape_arg(arg)
    end

    core.print(core.expand(cmd));
    return os.execute(core.expand(cmd))
end

-- Prints a string to standard output.
function core.print(str)
    str = str or ""
    print_raw(core.expand(str))
end

function core.ask(str)
    io.write(str .. " ")
    return io.read("l")
end

function core.ask_number(str)
    io.write(str .. " ")
    return io.read("n")
end

-- Parses an input table of options and merges it with a table of default values.
function core.options(given, defaults)
    if given == nil then
        return defaults
    end

    setmetatable(given, {
        __index = defaults
    })

    return given
end

-- Returns an iterator that iterates over each line in a string.
function string.lines(str)
    local remaining = str
    local next = ""
    local empty = false

    local function capture(line)
        next = line
        return ""
    end

    return function()
        if empty then
            return nil
        end

        next = nil
        remaining = remaining:gsub("(.-)\r?\n", capture, 1)

        if not next then
            empty = true
            next = remaining
        end

        return next
    end
end


-- Define some global function aliases.
export = core.export
exec = core.execute
print = core.print


return core
