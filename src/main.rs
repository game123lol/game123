mod entities;
mod map;
mod player;
mod systems;
use entities::{Item, Name, Position, Renderable};
use hecs::World;
use map::Map;
use player::new_player;
use std::collections::HashMap;
use systems::{
    move_player::move_player_system, pickup::run_pickup_system, render::run_render_system,
};
use tetra::{
    graphics::{self, scaling::ScreenScaler, Color, Rectangle, Texture},
    math::{Rect, Vec2},
    time::Timestep,
    window, Context, ContextBuilder, State, TetraError,
};

pub struct Game {
    world: World,
    resources: HashMap<String, Texture>,
    scaler: ScreenScaler,
}

impl State for Game {
    fn draw(&mut self, ctx: &mut Context) -> tetra::Result {
        let (w, h) = window::get_size(ctx);
        self.scaler.set_outer_size(w, h);
        graphics::set_canvas(ctx, self.scaler.canvas());
        graphics::clear(ctx, Color::rgb(0., 0., 0.));
        run_render_system(&self.world, ctx, &self.resources, self.scaler.inner_size());
        graphics::reset_canvas(ctx);
        graphics::clear(ctx, Color::rgb(0., 0., 0.));
        self.scaler.draw(ctx);
        Ok(())
    }
    fn update(&mut self, ctx: &mut Context) -> Result<(), TetraError> {
        move_player_system(&mut self.world, ctx);
        run_pickup_system(&mut self.world, ctx);
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
            "tileset_iso".into(),
            Texture::new(ctx, "assets/iso.png").unwrap(),
        );
        resources.insert(
            "items".into(),
            Texture::new(ctx, "assets/items.png").unwrap(),
        );
        let mut world = World::new();
        let map = Map::new(30, 20);
        world.spawn((map,));
        world.spawn(new_player());
        world.spawn((
            Position(Vec2::new(1, 1)),
            Renderable("person".into(), Rectangle::new(0., 0., 16., 16.)),
        ));
        world.spawn((
            Item,
            Name("item".into()),
            Renderable("items".into(), Rectangle::new(0., 0., 16., 16.)),
            Position(Vec2::new(2, 2)),
        ));
        world.spawn((Map::new(10, 10),));
        let scaler = ScreenScaler::new(
            ctx,
            500,
            500,
            500,
            500,
            graphics::scaling::ScalingMode::CropPixelPerfect,
        )
        .unwrap();
        Ok(Game {
            world,
            resources,
            scaler,
        })
    }
}

pub fn main() -> tetra::Result {
    ContextBuilder::new("S", 500, 500)
        .quit_on_escape(true)
        .resizable(true)
        .timestep(Timestep::Fixed(20.0))
        .build()?
        .run(Game::new)
}
