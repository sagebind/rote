local busted = {}


function busted.test(dir)
    local bustedCore = require 'busted.core'()
    local testFileLoader = require 'busted.modules.test_file_loader'(bustedCore)

    testFileLoader(dir, "_spec", {
        verbose = false,
        sort = nil,
        shuffle = nil,
        recursive = true,
        seed = bustedCore.randomseed
    })
end


return busted
