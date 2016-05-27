-- Module of tasks for Cargo, Rust's package manager.
local fs = require "fs"
local cargo = {}


-- Compile the current project.
function cargo.build(options)
    options = rote.options(options, {
        release = false,
        flags = {},
    })
    local args = {"cargo", "rustc"}

    if options.release then
        table.insert(args, "--release")
    end

    if options.verbose then
        table.insert(args, "--verbose")
    end

    if options.target then
        table.insert(args, "--target")
        table.insert(args, options.target)
    end

    if options.manifest then
        table.insert(args, "--manifest-path")
        table.insert(args, options.manifest)
    end

    if options.flags then
        table.insert(args, "--")
        for i, flag in ipairs(options.flags) do
            table.insert(args, flag)
        end
    end

    exec(table.unpack(args))
end

-- Gets metadata about the current Cargo package.
function cargo.info(manifest)
    manifest = manifest or "Cargo.toml"

    if not fs.exists(manifest) then
        return {}
    end

    local info = {
        authors = {},
    }
    local file = fs.get(manifest)

    for name in file:gmatch("name%s*=%s*\"([^\"]+)\"") do
        info.name = name
    end

    for version in file:gmatch("version%s*=%s*\"([^\"]+)\"") do
        info.version = version
    end

    for authors in file:gmatch("authors%s*=%s*%[([^%]]+)%]") do
        for author in authors:gmatch("\"([^\"]+)\",?") do
            table.insert(info.authors, author)
        end
    end

    for license in file:gmatch("license%s*=%s*\"([^\"]+)\"") do
        info.license = license
    end

    return info
end

-- Remove the target directory.
function cargo.clean()
    exec("cargo", "clean")
end

-- Build and execute src/main.rs.
function cargo.run()
    exec("cargo", "run")
end

-- Run the Cargo project tests.
function cargo.test()
    exec("cargo", "test")
end

-- Run the Cargo project benchmarks.
function cargo.bench()
    exec("cargo", "bench")
end

-- Update dependencies listed in Cargo.lock.
function cargo.update()
    exec("cargo", "update")
end


return cargo
