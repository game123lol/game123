use std::collections::HashMap;

use hecs::World;
use map::Map;
use systems::{move_player::move_player_system, render::run_render_system};
use tetra::{
    graphics::{self, Color, Rectangle, Texture},
    math::Vec2,
    Context, ContextBuilder, State, TetraError,
};

mod systems;

mod map;

#[derive(Debug)]
struct Renderable(String, Rectangle);

struct Player;

struct Position(Vec2<f32>);

struct Game {
    world: World,
    resources: HashMap<String, Texture>,
}

impl State for Game {
    fn draw(&mut self, ctx: &mut Context) -> tetra::Result {
        graphics::clear(ctx, Color::rgb(0.392, 0.584, 0.929));
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
        let mut world = World::new();
        world.spawn((
            Position(Vec2::new(1.0, 1.0)),
            Renderable("person".into(), Rectangle::new(0., 0., 16., 16.)),
            Player,
        ));
        world.spawn((
            Position(Vec2::new(1.0, 1.0)),
            Renderable("person".into(), Rectangle::new(0., 0., 16., 16.)),
        ));
        world.spawn((Map::new(10, 10),));
        Ok(Game { world, resources })
    }
}
pub fn main() -> tetra::Result {
    ContextBuilder::new("S", 500, 500)
        .quit_on_escape(true)
        .build()?
        .run(Game::new)
}
