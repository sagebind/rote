-- Module of tasks for Docker, the Linux container suite.

docker = {}

function docker.build(options)
    local options = rote.options(options, {
        path = "."
    })
    local args = {"docker build"}

    if options.tag then
        table.insert(args, "-t " .. options.tag)
    end

    table.insert(args, options.path)

    exec(unpack(args))
end

return docker
