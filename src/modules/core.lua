rote = {}

-- Expands global and environment variables inside a given string.
function rote.expand(string)
    return string.gsub(string, "$(%w+)", function (name)
        return os.getenv(name) or _G[name] or ""
    end)
end

-- Executes a shell command.
function rote.execute(cmd)
    return os.execute(rote.expand(cmd))
end

-- Prints a string to standard output.
function rote.echo(string)
    string = string or ""
    io.write(rote.expand(string), "\n")
end

-- Define some global function aliases.
exec = rote.execute
echo = rote.echo

return rote
