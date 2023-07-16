mod map;
mod player;
mod systems;
use hecs::{Entity, World};
use map::Map;
use player::new_player;
use std::collections::HashMap;
use systems::{
    move_player::move_player_system,
    render::{run_map_render_system_iso, run_render_system},
};
use tetra::{
    graphics::{self, Color, Rectangle, Texture},
    math::Vec2,
    time::Timestep,
    Context, ContextBuilder, State, TetraError,
};

#[derive(Debug)]
pub struct Renderable(String, Rectangle);

pub struct Player;

pub struct Position(Vec2<i32>);

pub struct Item;

pub struct ContainsBy(Entity);

pub struct Game {
    world: World,
    resources: HashMap<String, Texture>,
}

pub struct Sight(Vec<bool>);

impl State for Game {
    fn draw(&mut self, ctx: &mut Context) -> tetra::Result {
        graphics::clear(ctx, Color::rgb(0., 0., 0.));
        run_map_render_system_iso(&self.world, ctx, &self.resources);
        run_render_system(&self.world, ctx, &self.resources);
        Ok(())
    }
    fn update(&mut self, ctx: &mut Context) -> Result<(), TetraError> {
        move_player_system(&mut self.world, ctx);
        Ok(())
    }
}

impl Game {
    fn new(ctx: &mut Context) -> tetra::Result<Game> {
        let mut resources = HashMap::new();
        resources.insert(
            "person".into(),
            Texture::new(ctx, "assets/person.png").unwrap(),
        );
        resources.insert(
            "tileset".into(),
            Texture::new(ctx, "assets/Tileset.png").unwrap(),
        );
        resources.insert(
            "tileset_iso".into(),
            Texture::new(ctx, "assets/iso.png").unwrap(),
        );
        let mut world = World::new();
        let map = Map::new(30, 20);
        world.spawn((map,));
        world.spawn(new_player());
        world.spawn((
            Position(Vec2::new(1, 1)),
            Renderable("person".into(), Rectangle::new(0., 0., 16., 16.)),
        ));
        world.spawn((Map::new(10, 10),));
        Ok(Game { world, resources })
    }
}

pub fn main() -> tetra::Result {
    ContextBuilder::new("S", 500, 500)
        .quit_on_escape(true)
        .timestep(Timestep::Fixed(20.0))
        .build()?
        .run(Game::new)
}
