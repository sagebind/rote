-- Module for PHP build and maintennance tasks.

php = {}


function php.server(port, host, dir)
    local port = port or 8080
    local host = host or "127.0.0.1"

    local args = {"php", "-S", port .. ":" .. host}

    if dir then
        table.insert(args, "-t")
        table.insert(args, dir)
    end

    exec(table.unpack(args))
end


return php
