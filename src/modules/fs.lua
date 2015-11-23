-- Module of file system-related tasks.

fs = {}

-- Checks if a file exists and is readable.
function fs.exists(path)
    local file = io.open(path, "rb")
    if file then
        file:close()
    end
    return file ~= nil
end

-- Moves a file from one name to another.
function fs.move(source, destination)
    return os.rename(source, destination)
end

-- Copies a file to another location.
function fs.copy(source, destination)
    local content = fs.get(source)
    fs.put(destination, content)
end

-- Removes a file.
function fs.remove(path)
    os.remove(path)
end

-- Creates a directory.
function fs.mkdir(path)
    exec("mkdir", path)
end

-- Reads an entire file and returns its contents.
function fs.get(path)
    local file, err = io.open(path, "rb")
    if err then
        error(err)
    end

    local content = file:read("a")
    file:close()
    return content
end

-- Puts a string into the contents of a file.
function fs.put(path, contents)
    local file, err = io.open(path, "wb")
    if err then
        error(err)
    end

    file:write(content)
    file:close()
end

-- Combines the contents of two or more files into a new, single file.
function fs.combine(sources, destination)
    if not sources then
        return nil
    end

    local dest_file, err = io.open(destination, "wb")
    if err then
        error(err)
    end

    -- Loop over each source and pipe into the destination.
    for i, source in ipairs(sources) do
        local src_file, err = io.open(source, "rb")

        if err then
            error(err)
        end

        dest_file:write(src_file:read("a"))
        src_file:close()
    end

    dest_file:close()
end

return fs
