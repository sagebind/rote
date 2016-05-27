local install = {}


function install.binary(binary)
    if os == "unix" then
        exec("install", "-s", binary)
    end
end


return install
