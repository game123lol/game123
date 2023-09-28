mod components;
mod map;
mod player;
mod resources;
mod systems;
mod tests;
use components::{Item, Name, Position, Renderable};
use hecs::World;
use map::Map;
use player::new_player;
use resources::Resources;
use std::env;
use systems::{
    fov_compute::run_fov_compute_system, move_player::move_player_system,
    render::run_render_system_fov,
};
use tetra::{
    graphics::{self, scaling::ScreenScaler, Color},
    math::Vec2,
    window, Context, ContextBuilder, State, TetraError,
};

pub struct Game {
    world: World,
    resources: Resources,
    scaler: ScreenScaler,
}

impl State for Game {
    fn draw(&mut self, ctx: &mut Context) -> tetra::Result {
        let (w, h) = window::get_size(ctx);
        self.scaler.set_outer_size(w, h);
        graphics::set_canvas(ctx, self.scaler.canvas());
        graphics::clear(ctx, Color::rgb(0., 0., 0.));
        run_render_system_fov(&self.world, ctx, &self.resources, self.scaler.inner_size());
        graphics::reset_canvas(ctx);
        graphics::clear(ctx, Color::rgb(0., 0., 0.));
        self.scaler.draw(ctx);
        Ok(())
    }
    fn update(&mut self, ctx: &mut Context) -> Result<(), TetraError> {
        move_player_system(&mut self.world, ctx);
        run_fov_compute_system(&mut self.world);
        Ok(())
    }
}

impl Game {
    fn new(ctx: &mut Context) -> tetra::Result<Game> {
        let exe_path = env::current_exe().expect("Ты ебанутый? Ты что там делаешь?");
        let mut assets_path = exe_path
            .parent()
            .and_then(|p| p.parent())
            .map(|p| p.join("assets"))
            .unwrap();
        if !assets_path.exists() {
            assets_path = env::current_dir().unwrap().join("assets"); //мммм а пахне як
        }
        let resources = Resources::load(ctx, &assets_path);
        let mut world = World::new();
        let map = Map::new();
        world.spawn((map,));
        world.spawn(new_player());
        world.spawn((
            Item, //TODO: убрать эту отсебятину и сделать норм генератор предметов
            Name("item".into()),
            Renderable("item".into()),
            Position(Vec2::new(2, 2)),
        ));
        let scaler = ScreenScaler::new(
            ctx,
            1000,
            1000,
            1000,
            1000,
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
        .build()?
        .run(Game::new)
}
