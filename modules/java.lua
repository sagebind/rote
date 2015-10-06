java = {}

function java.binary(options)
    local options = rote.options(options, {
        dest = ".",
        paths = {"."},
        warnings = true,
        debug = true,
        compiler = "javac",
        encoding = "utf-8"
    })

    local compiler_flags = {
        "-verbose",
        "-encoding", options.encoding,
        "-cp", table.concat(options.paths, ":"),
        "-d", options.dest
    }

    if not options.warnings then
        table.insert(compiler_flags, "-nowarn")
    end

    if options.debug then
        table.insert(compiler_flags, "-g")
    else
        table.insert(compiler_flags, "-g:none")
    end

    for i, file in ipairs({glob(options.srcs)}) do
        table.insert(compiler_flags, file)
    end

    exec(options.compiler .. " " .. table.concat(compiler_flags, " "))
end

return java
