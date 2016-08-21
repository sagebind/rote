json = require "json"


assert(json.parse)
assert(json.stringify)

do -- json.parse
    assert(json.parse("null") == nil)
    assert(json.parse("true") == true)
    assert(json.parse("42") == 42)
    assert(json.parse("\"marvin\"") == "marvin")

    local result = json.parse([[{
        "code": 200,
        "success": true,
        "payload": {
            "features": [
                "awesome",
                "easyAPI",
                "lowLearningCurve"
            ]
        }
    }]])

    assert(type(result) == "table")
    assert(result.code == 200)
    assert(result.success == true)
    assert(type(result.payload) == "table")
    assert(type(result.payload.features) == "table")
    assert(result.payload.features[1] == "awesome")
    assert(result.payload.features[2] == "easyAPI")
    assert(result.payload.features[3] == "lowLearningCurve")
end

do -- json.stringify
    assert(json.stringify(nil) == "null")
    assert(json.stringify(true) == "true")
    assert(json.stringify(42) == "42")
    assert(json.stringify("marvin") == "\"marvin\"")
    assert(json.stringify({
        life = 42
    }) == "{\"life\":42}")
end
