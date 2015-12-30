-- Module of tasks for Cargo, Rust's package manager.

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

    exec(table.unpack(args))
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
