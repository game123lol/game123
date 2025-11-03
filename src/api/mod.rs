// hi!

use std::{
    collections::HashSet,
    fs,
    sync::{Arc, Mutex},
};

use hecs::{Entity, World};
use mlua::{Function, Lua, ObjectLike, Table, Value};

use crate::{
    components::Position,
    hasher,
    inventory::Inventory,
    mob::{Log, Mob},
    player::Player,
    resources::Resources,
    systems::{fov_compute::Sight, memory::MapMemory, pathfinding::Pathfinder},
};

#[derive(Debug)]
pub struct Mod {
    name: String,
    init: String,
}

pub struct LuaApi {
    lua: Lua,
    mods: Vec<Mod>,
}

pub struct LuaEntity(pub Entity);

fn spawn_item_lua(
    world: &mut World,
    resources: &Resources,
    name: &str,
    pos: [i32; 3],
) -> Result<(), mlua::Error> {
    let item = resources.template_item(name);
    world.spawn(item.into_map_entity(&Position::new(pos[0], pos[1], pos[2])));
    Ok(())
}

fn spawn_entity_lua(
    world: &mut World,
    resources: &Resources,
    name: &str,
    table: Table,
) -> Result<Entity, mlua::Error> {
    let mut eb = resources.template_entity(name);
    if let Ok(overrides) = table.get::<Table>("overrides") {
        for (key, value) in overrides.pairs::<String, Value>().flatten() {
            match key.as_str() {
                "mob" => {
                    eb.add(Mob);
                }
                "pathfinder" => {
                    eb.add(Pathfinder);
                }
                "map_memory" => {
                    eb.add(MapMemory::new());
                }
                "inventory" => {
                    eb.add(Inventory::new());
                }
                "log" => {
                    eb.add(Log(String::new()));
                }
                "player" => {
                    eb.add(Player);
                }
                "sight" => {
                    if let Value::Integer(a) = value {
                        eb.add(Sight(a as u32, HashSet::with_hasher(hasher())));
                    }
                }
                a => {
                    dbg!("sdfjkghwleghwpoegh так быть не должно");
                }
            }
        }
    }
    if table.get::<i32>("x").is_ok()
        && table.get::<i32>("y").is_ok()
        && table.get::<i32>("z").is_ok()
    {
        let x = table.get::<i32>("x")?;
        let y = table.get::<i32>("y")?;
        let z = table.get::<i32>("z")?;
        eb.add(Position::new(x, y, z));
    }
    let e = world.spawn(eb.build());
    Ok(e)
}

impl LuaApi {
    pub async fn init(world: &mut World, resources: &mut Resources) -> Self {
        let lua = Lua::new();
        let mut mods = Vec::new();
        let mut mod_paths = std::fs::read_dir("./mods").unwrap().flatten();
        let world = Arc::new(Mutex::new(world));

        for game_mod in &mut mod_paths {
            println!(
                "loading {} mod",
                game_mod.file_name().into_string().unwrap()
            );
            resources.load(game_mod.path().as_path()).await;
            let init_lua = fs::read_to_string(game_mod.path().join("init.lua")).unwrap();
            mods.push(Mod {
                name: game_mod.file_name().into_string().unwrap(),
                init: init_lua,
            });
        }
        lua.sandbox(true).unwrap();

        let globals = lua.globals();
        lua.scope(|s| {
            let spawn_entity_fn = s
                .create_function_mut(|_, (name, table): (String, _)| {
                    let binding = world.clone();
                    let mut world = binding.lock().unwrap();
                    spawn_entity_lua(*world, resources, name.as_str(), table)
                        .map(|x| lua.create_any_userdata(x))
                        .flatten()
                })
                .unwrap();
            globals.set("spawn_entity", spawn_entity_fn).unwrap();
            let spawn_item_fn = s
                .create_function_mut(|_, (name, x, y, z): (String, _, _, _)| {
                    let binding = world.clone();
                    let mut world = binding.lock().unwrap();
                    spawn_item_lua(*world, resources, name.as_str(), [x, y, z])
                })
                .unwrap();
            globals.set("spawn_item", spawn_item_fn).unwrap();

            for game_mod in &mods {
                lua.load(&game_mod.init).exec().unwrap();
                let init_fn = globals.get::<Function>("world_init").unwrap();
                init_fn.call::<()>(()).unwrap();
            }
            Ok(())
        })
        .unwrap();
        LuaApi { lua, mods }
    }
}
