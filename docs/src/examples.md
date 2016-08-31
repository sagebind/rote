# Examples

```lua
require "cargo"
default "debug"


desc("Build a debug binary")
task("debug", function()
    print "Building debug version..."
    cargo.build()
end)

desc("Build a release binary")
task("release", function()
    cargo.build {
        release = true
    }
end)

desc("Clean the project directory")
task("clean", function()
    cargo.clean()
end)

desc("Runs tests")
task("test", function()
    for _, file in ipairs(glob("tests/*.lua")) do
        print("[" .. file .. "]")
        local success, err = pcall(function()
            return dofile(file)
        end)

        if not success then
            print "FAIL!"
            print("Reason: " .. err)
        else
            print "PASS"
        end
    end
end)
```



C++ example:

```lua
desc("Build a C++ program")
task("main", {"target/main"})

rule("target/main", cpp.binary {
    bin = "target/main",
    srcs = glob("src/**"),
    libs = {"glibc"},
})

cpp.binary("target/main", {}, {
    srcs = glob("src/**"),
    libs = {"glibc"},
})
```
