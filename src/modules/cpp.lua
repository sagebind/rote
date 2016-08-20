-- Module for generating C/C++ build tasks.
local cpp = {}


function cpp.binary(options)
    options = rote.merge({
        standard = "c++11",
        debug = true,
        includes = {},
        opt_level = 0,
        macros = {},
    }, options)

    if not options.name then
        error("name must be specified")
    end

    if not options.srcs then
        error("srcs must be specified")
    end

    local compiler = CC or "g++"
    local linker = CC or "g++"

    local compiler_flags = {"-Wall", "--std=" .. string.lower(options.standard)}
    local linker_flags = {}

    -- Add include paths.
    for _, include in ipairs(options.includes) do
        table.insert(compiler_flags, "-I" .. include)
    end

    if options.debug then
        table.insert(compiler_flags, "-g")
    end

    -- Set optimization level.
    table.insert(compiler_flags, ("-O%d"):format(options.opt_level))
    table.insert(linker_flags, ("-O%d"):format(options.opt_level))

    -- Pass all macro definitions to the preprocessor. Macros will be set to `1`
    -- if given as a truthy, non-string value.
    for macro, def in ipairs(options.macros) do
        if type(def) == "string" then
            table.insert(compiler_flags, "-D" .. macro .. "=" .. def)
        elseif def then
            table.insert(compiler_flags, "-D" .. macro)
        end
    end

    -- Create individual rules for each source file.
    local object_files = {}
    for i, file in ipairs(options.srcs) do
        local object_file = file .. ".o"
        table.insert(object_files, object_file)

        rote.create_rule(object_file, function(object_file)
            local args = {}
            for _, flag in ipairs(compiler_flags) do
                table.insert(args, flag)
            end
            table.insert(args, "-c")
            table.insert(args, "-o")
            table.insert(args, object_file)
            table.insert(args, file)

            exec(compiler, table.unpack(args))
        end)
    end

    -- Create a rule for the final binary.
    rote.create_rule(options.name, object_files, function(name)
        local args = {}
        for _, flag in ipairs(linker_flags) do
            table.insert(args, flag)
        end
        table.insert(args, "-o")
        table.insert(args, name)
        for _, object_file in ipairs(object_files) do
            table.insert(args, object_file)
        end

        exec(linker, table.unpack(args))
    end)
end


return cpp
