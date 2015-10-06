-- Taken from http://www.cs.tufts.edu/~nr/drop/lua/tabutil.lua
-- by Norman Ramsey

local setmetatable, getmetatable, table, type, math, unpack
    = setmetatable, getmetatable, table, type, math, unpack

function table.of_generator(f, ...)
  local l = { }
  for x in f(...) do table.insert(l, x) end
  return l
end

function table.shift(t, n)
  n = n or 1
  for i = 1, n do
    table.remove(t, 1)
  end
end

function table.random(l, n)
  n = n or table.getn(l)
  assert(n > 0, 'random element from empty list')
  return l[math.random(n)]
end

function table.sorted_keys(t, lt)
  local u = { }
  for k in pairs(t) do table.insert(u, k) end
  table.sort(u, lt)
  return u
end

function table.randomize(l)
  u = { }
  local n = table.getn(l)
  for i = 1, n do
    u[i] = l[i]
  end
  for i = 1, n do
    j = math.random(n)
    u[i], u[j] = u[j], u[i]
  end
  return u
end

function table.filter(f, l)
  local u = { }
  for _, v in ipairs(l) do
    if f(v) then
      table.insert(u, v)
    end
  end
  return u
end

function table.map(f, l)
  local u = { }
  for k, v in pairs(l) do
    u[k] = f(v, k)
  end
  return u
end

local function pack(...) return { ... } end

function table.fold(f, t, ...)
  local z = pack(...)
  for k, v in pairs(t) do
    z = pack(f(k, v, unpack(z)))
  end
  return unpack(z)
end

function table.forall(f, l)
  for _, v in ipairs(l) do
    if not f(v) then
      return false
    end
  end
  return true
end

function table.exists(f, l)
  for _, v in ipairs(l) do
    if f(v) then
      return true
    end
  end
  return false
end

function table.find(f, l)
  for _, v in ipairs(l) do
    if f(v) then
      return v
    end
  end
  return nil
end

function table.index_of(l, v)
  for i = 1, table.getn(l) do
    if l[i] == v then return i end
  end
  return nil
end

function table.invert(t)
  local u = { }
  for k, v in pairs(t) do u[v] = k end
  return u
end

function table.set(t) -- set of list
  local u = { }
  for _, v in ipairs(t) do u[v] = true end
  return u
end

function table.subset(l, r) -- every key in l is in r
  for k in pairs(l) do if r[k] == nil then return false end end
  return true
end

function table.numpairs(t)
  local n = 0
  for _ in pairs(t) do n = n + 1 end
  return n
end

function table.union(...)
  local u = { }
  for _, t in ipairs { ... } do
    for k in pairs(t) do
      u[k] = true
    end
  end
  return u
end

function table.elements(t) --- list of set
  local u = { }
  for v in pairs(t) do table.insert(u, v) end
  return u
end

function table.values(t) --- values in table as list
  local u = { }
  for _, v in pairs(t) do table.insert(u, v) end
  return u
end

function table.copy(t)
  local u = { }
  for k, v in pairs(t) do u[k] = v end
  return setmetatable(u, getmetatable(t))
end

do
  local default_meta = { __index = function(t) return t['*'] or t.default end }
  function table.defaulting(t, defaultkey)
    t = t or { }
    defaultkey = defaultkey or '*'
    local om = getmetatable(t)
    if om then
      assert(om.__index == nil, "Defaulting table already has __index method")
      local m = { }
      for k, v in pairs(om) do m[k] = v end
      m.__index = function(t) return t[defaultkey] or t.default end
      setmetatable(t, m)
    elseif defaultkey == '*' then
      setmetatable(t, default_meta)
    else
      setmetatable(t, { __index = function(t) return t[defaultkey] or t.default end })
    end
    return t
  end
end

do
  local meta = { __index = function(t, k) local u = {} t[k] = u return u end }
  function table.of_tables(t)
    t = t or { }
    local om = getmetatable(t)
    if om then
      assert(om.__index == nil, "Defaulting table already has __index method")
      local m = { }
      for k, v in pairs(om) do m[k] = v end
      m.__index = meta.__index
      setmetatable(t, m)
    else
      setmetatable(t, meta)
    end
    return t
  end
end

do
  local meta = { __index = function(t, k) t[k] = 0 return 0 end }
  function table.of_zeroes(t)
    t = t or { }
    local om = getmetatable(t)
    if om then
      assert(om.__index == nil, "Defaulting table already has __index method")
      local m = { }
      for k, v in pairs(om) do m[k] = v end
      m.__index = meta.__index
      setmetatable(t, m)
    else
      setmetatable(t, meta)
    end
    return t
  end
end

-------------
--- for lexicographic sorting

local function listlt(l1, l2, lt)
  for i = 1, table.getn(l1) do
    local v1, v2 = l1[i], l2[i]
    if lt(v1, v2) then return true
    elseif lt(v2, v1) then return false
    end
  end
  return false
end

function table.lt(l1, l2, lt) -- must be same length
  if lt then return listlt(l1, l2, lt) end
  for i = 1, table.getn(l1) do
    local v1, v2 = l1[i], l2[i]
    if v1 ~= v2 then
      if type(v1) == 'boolean' then
        return not v1 and v2
      elseif type(v1) == 'nil' then
        return true
      elseif type(v2) == 'nil' then
        return false
      else
        return v1 < v2
      end
    end
  end
  return false
end

function table.gt(l1, l2, lt) return table.lt(l2, l1, lt) end


return table
