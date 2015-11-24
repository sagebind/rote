WORLD = "Earth"
assert(rote.expand("Hello $WORLD!") == "Hello Earth!")


assert(rote.escape_arg("-f") == "'-f'")
assert(rote.escape_arg("foo bar") == "'foo bar'")
assert(rote.escape_arg("'foo' bar") == "'\\'foo\\' bar'")


assert(rote.options(nil, {}) ~= nil)
assert(rote.options({
    foo = "bar"
}, {}).foo == "bar")
assert(rote.options({}, {
    foo = "bar"
}).foo == "bar")
assert(rote.options({
    foo = "bar"
}, {
    foo = "baz"
}).foo == "bar")
