WORLD = "Earth"
assert(rote.expand("Hello $WORLD!") == "Hello Earth!")

assert(rote.merge(nil, {}) ~= nil)
assert(rote.merge({
    foo = "bar"
}, {}).foo == "bar")
assert(rote.merge({}, {
    foo = "bar"
}).foo == "bar")
assert(rote.merge({
    foo = "bar"
}, {
    foo = "baz"
}).foo == "baz")
