-- Module for Git VCS maintennance tasks.

git = {}

function git.add(pattern)
    exec("git add", pattern)
end

function git.checkout(branch)
    exec("git checkout", branch or "")
end

function git.commit(message)
    exec("git commit", "-m" .. message)
end

function git.merge(branch)
    exec("git merge", branch or "")
end

function git.push(origin, branch)
    exec("git push", origin or "", branch or "")
end

function git.pull(origin, branch)
    exec("git pull", origin or "", branch or "")
end

function git.tag(name)
    exec("git tag", name)
end

return git
