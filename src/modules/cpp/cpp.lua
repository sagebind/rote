-- Module for C/C++ build tasks.
local cpp = {}


function cpp.binary(options)
    options = rote.options(options, {
        out = "bin/a.out",
        standard = "c++11",
        debug = true,
        macros = {},
    })

    local compiler = "g++"
    local linker = "g++"

    local compiler_flags = {"-Wall", "--std=" .. string.lower(options.standard)}
    local linker_flags = {}

    -- Pass all macro definitions to the preprocessor. Macros will be set to `1`
    -- if given as a truthy, non-string value.
    for name, def in ipairs(options.macros) do
        if type(def) == "string" then
            table.insert(compiler_flags, "-D" .. name .. "=" .. def)
        elseif def then
            table.insert(compiler_flags, "-D" .. name)
        end
    end

    if options.debug then
        table.insert(compiler_flags, "-g")
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
