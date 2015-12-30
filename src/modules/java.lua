-- Module for Java build tasks.

java = {}


function java.binary(options)
    options = rote.options(options, {
        dest = ".",
        paths = {"."},
        warnings = true,
        debug = true,
        compiler = "javac",
        encoding = "utf-8",
        mainClass = "Main"
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

    for i, file in ipairs(glob(options.srcs)) do
        table.insert(compiler_flags, file)
    end

    exec(options.compiler .. " " .. table.concat(compiler_flags, " "))
end

function java.jar(options)
    local options = rote.options(options, {
        out = "out.jar",
        main_class = "org.myapp.Main",
        manifest = false
    })

    local jar_flags = {"-c", "-v", "-f", options.out}

    if options.mainClass then
        table.insert(jar_flags, "-e")
        table.insert(jar_flags, options.mainClass)
    end

    if options.manifest then
        table.insert(jar_flags, "-m")
        table.insert(jar_flags, options.manifest)
    end

    exec("jar " .. table.concat(jar_flags, " ") .. options.srcs)
end


return java
