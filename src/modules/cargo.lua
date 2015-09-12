cargo = {}

function cargo.build(params)
    if params and params.release then
        exec "cargo build --release --verbose"
    else
        exec "cargo build --verbose"
    end
end

function cargo.clean()
    exec "cargo clean"
end

function cargo.test()
    exec "cargo test"
end

return cargo
