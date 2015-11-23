-- Module of tasks for Cargo, Rust's package manager.

cargo = {}

function cargo.build(params)
    local args = {"cargo build --verbose"}

    if params and params.release then
        table.insert(args, "--release")
    end

    exec(table.unpack(args))
end

function cargo.clean()
    exec "cargo clean"
end

function cargo.test()
    exec "cargo test"
end

return cargo
