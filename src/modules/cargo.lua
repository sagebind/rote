-- Module of tasks for Cargo, Rust's package manager.
require "fs"

cargo = {}


-- Compile the current project.
function cargo.build(options)
    options = rote.options(options, {
        release = false
    })
    local args = {"cargo", "build", "--verbose"}

    if options.release then
        table.insert(args, "--release")
    end

    if options.target then
        table.insert(args, "--target")
        table.insert(args, options.target)
    end

    exec(table.unpack(args))
end

-- Gets metadata about the current Cargo package.
function cargo.info()
    if not fs.exists("Cargo.toml") then
        return {}
    end

    local info = {
        authors = {},
    }
    local file = fs.get("Cargo.toml")

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
    exec "cargo clean"
end

-- Build and execute src/main.rs.
function cargo.run()
    exec "cargo run"
end

-- Run the Cargo project tests.
function cargo.test()
    exec "cargo test"
end

-- Run the Cargo project benchmarks.
function cargo.bench()
    exec "cargo bench"
end

-- Update dependencies listed in Cargo.lock.
function cargo.update()
    exec "cargo update"
end


return cargo
