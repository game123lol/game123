function world_init()
  local a =spawn_entity("human", {
    x = 1, y = 1, z = 0,
    overrides = {
      player = {}
    }
  })
  local a =spawn_entity("human", {
    x = 10, y = 10, z = 0,
    overrides = {
    }
  })
  -- spawn_entity("nettle", {
  --   x = 10, y = 10, z = 0
  -- })
  spawn_item("knife", 2, 2, 0)
  spawn_item("knife", 2, 3, 0)
  spawn_item("knife", 2, 4, 0)
  spawn_item("bat", 3, 2, 0)
  spawn_item("bat", 3, 3, 0)
  spawn_item("bat", 3, 4, 0)
end
