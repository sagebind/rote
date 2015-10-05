cpp = {}

function cpp.binary(options)
    local options = options or {}

    local compiler = "g++"
    local linker = "g++"
    local standard = string.lower(options.standard or "c++11")
    local debug = options.debug or true
    local out = options.out or "bin/a.out"

    local compiler_flags = "-Wall --std=" .. standard
    local linker_flags = ""

    if debug then
        compiler_flags = compiler_flags .. " -g"
    end

    local obj_files = {}
    for i, file in ipairs({glob(options.srcs)}) do
        local obj = file .. ".o"
        table.insert(obj_files, obj)

        exec(compiler .. " " .. compiler_flags .. " -c -o " .. obj .. " " .. file)
    end

    exec(linker .. " " .. linker_flags .. " -o " .. out .. " " .. table.concat(obj_files, " "))
end

return cpp
