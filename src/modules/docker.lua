-- Module of tasks for Docker, the Linux container suite.

docker = {}


-- Builds an image from a Dockerfile.
function docker.build(options)
    local options = rote.options(options, {
        path = ".",
        cache = true,
    })
    local args = {"docker", "build"}

    if options.file then
        table.insert(args, "-f")
        table.insert(args, options.file)
    end

    if options.tag then
        table.insert(args, "-t")
        table.insert(args, options.tag)
    end

    if not options.cache then
        table.insert(args, "--no-cache=true")
    end

    table.insert(args, options.path)

    exec(table.unpack(args))
end

-- Runs a command in a new Docker container.
function docker.run(options)
    local options = rote.options(options, {
        image = nil,
        environment = {},
        expose = {},
        labels = {},
        ports = {},
        volumes = {},
    })
    local args = {"docker", "run"}

    if not options.image then
        return nil
    end

    for name, value in ipairs(options.environment) do
        table.insert(args, "-e")
        table.insert(args, name .. "=" .. value)
    end

    for i, port in ipairs(options.expose) do
        table.insert(args, "--expose=" .. port)
    end

    if options.detach then
        table.insert(args, "-d")
    end

    if options.hostname then
        table.insert(args, "--hostname=" .. options.hostname)
    end

    if options.interactive then
        table.insert(args, "-i")
    end

    for name, value in ipairs(options.labels) do
        table.insert(args, "-l")
        table.insert(args, name .. "=" .. value)
    end

    if options.name then
        table.insert(args, "--name=" .. options.name)
    end

    for i, port in ipairs(options.ports) do
        table.insert(args, "-p")
        table.insert(args, port)
    end

    if options.remove then
        table.insert(args, "--rm=true")
    end

    if options.restart ~= "no" then
        table.insert(args, "--restart=" .. options.restart)
    end

    if options.tty then
        table.insert(args, "-t")
    end

    for i, volume in ipairs(options.volumes) do
        table.insert(args, "-v")
        table.insert(args, volume)
    end

    for i, container in ipairs(options.volumes_from) do
        table.insert(args, "--volumes-from=" .. container)
    end

    table.insert(args, options.image)

    if options.command then
        table.insert(args, command)
    end

    exec(table.unpack(args))
end


return docker
