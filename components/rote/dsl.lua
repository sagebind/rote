__next_description = nil


function task(name, deps, callback)
    if type(deps) == "function" then
        callback = deps
        deps = {}
    end

    rote.create_task(name, __next_description, deps, callback)
    __next_description = nil
end

function rule(pattern, deps, callback)
    if type(deps) == "function" then
        callback = deps
        deps = {}
    end

    rote.create_rule(pattern, __next_description, deps, callback)
    __next_description = nil
end

function desc(description)
    __next_description = description
end

function default(name)
    rote.set_default_task(name)
end
