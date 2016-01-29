-- Module for Git VCS maintennance tasks.
local git = {}


-- Add file contents to the index.
function git.add(pattern)
    exec("git add", pattern)
end

-- Switch branches or restore working tree files.
function git.checkout(branch)
    exec("git checkout", branch or "")
end

-- Record changes to the repository.
function git.commit(message)
    exec("git commit", "-m" .. message)
end

-- Join two or more development histories together.
function git.merge(branch)
    exec("git merge", branch or "")
end

-- Update remote refs along with associated objects.
function git.push(origin, branch)
    exec("git push", origin or "", branch or "")
end

-- Fetch from and integrate with another repository or a local branch.
function git.pull(origin, branch)
    exec("git pull", origin or "", branch or "")
end

-- Create, list, delete or verify a tag object signed with GPG.
function git.tag(name)
    exec("git tag", name)
end


return git
