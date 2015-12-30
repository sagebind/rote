-- Module for C/C++ build tasks.

cpp = {}


function cpp.binary(options)
    options = rote.options(options, {
        out = "bin/a.out",
        standard = "c++11",
        debug = true
    })

    local compiler = "g++"
    local linker = "g++"

    local compiler_flags = "-Wall --std=" .. string.lower(options.standard)
    local linker_flags = ""

    if options.debug then
        compiler_flags = compiler_flags .. " -g"
    end

    local obj_files = {}
    for i, file in ipairs({glob(options.srcs)}) do
        local obj = file .. ".o"
        table.insert(obj_files, obj)

        exec(compiler .. " " .. compiler_flags .. " -c -o " .. obj .. " " .. file)
    end

    exec(linker .. " " .. linker_flags .. " -o " .. options.out .. " " .. table.concat(obj_files, " "))
end


return cpp
