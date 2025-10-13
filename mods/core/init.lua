function world_init()
  spawn_entity("human", {
    x = 1, y = 1, z = 0,
    overrides = {
      player = {}
    }
  })
  spawn_entity("nettle", {
    x = 10, y = 10, z = 0
  })
  spawn_item("knife", 2, 2, 0)
  spawn_item("knife", 2, 3, 0)
  spawn_item("knife", 2, 4, 0)
end
