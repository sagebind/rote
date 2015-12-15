fs = require "fs"


assert(fs.exists("tests") == true)
assert(fs.exists("tests/fs.lua") == true)
assert(fs.exists("tests/fs.looa") == false)

fs.mkdir("tests/fstestdir")
assert(fs.exists("tests/fstestdir") == true)
fs.remove("tests/fstestdir")
assert(fs.exists("tests/fstestdir") == false)

assert(fs.get("tests/fixtures/fs1.in") == "hello\n")

fs.put("tests/fixtures/fs1.out", "hello\n")
assert(fs.get("tests/fixtures/fs1.out") == "hello\n")

fs.append("tests/fixtures/fs1.out", "world\n")
assert(fs.get("tests/fixtures/fs1.out") == "hello\nworld\n")

fs.combine(
    {
        "tests/fixtures/fs1.in",
        "tests/fixtures/fs2.in"
    },
    "tests/fixtures/fs2.out"
)
assert(fs.get("tests/fixtures/fs2.out") == "hello\nworld\n")

fs.remove("tests/fixtures/fs1.out")
assert(fs.exists("tests/fixtures/fs1.out") == false)
fs.remove("tests/fixtures/fs2.out")
assert(fs.exists("tests/fixtures/fs2.out") == false)
