local sass = {}


function sass.compile(options)
    options = core.options(options, {
        input = nil,
        output = nil,
        style = "compressed",
        cache = true,
        trace = true,
    })

    local args = {"sass", "--style", options.style}

    if not options.cache then
        table.insert(args, "--no-cache")
    end

    if options.trace then
        table.insert(args, "--trace")
    end

    table.insert(args, input)
    table.insert(args, output)

    exec(table.unpack(args))
end


return sass
