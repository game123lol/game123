mod components;
mod map;
mod player;
mod resources;
mod systems;
mod tests;
use components::{Item, Name, Position, Renderable};
use egui_tetra::{
    egui::{self, Pos2},
    StateWrapper,
};
use hecs::World;
use map::WorldMap;
use player::new_player;
use resources::Resources;
use std::env;
use systems::{
    fov_compute::FovComputeSystem, memory::MemorySystem, move_player::MovePlayerSystem,
    render::RenderSystem, GameSystem, WorldSystem,
};
use tetra::{
    graphics::{self, scaling::ScreenScaler, Color},
    math::Vec2,
    window, Context, ContextBuilder,
};

type GameSystems = Vec<Box<dyn GameSystem>>;
type WorldSystems = Vec<Box<dyn WorldSystem>>;

type Type = World;

enum UIState {
    No,
    Inventory(Vec<String>),
    Debug,
}

pub struct Game {
    world: Type,
    resources: Resources,
    scaler: ScreenScaler,
    game_systems: GameSystems,
    world_systems: WorldSystems,
    ui_state: UIState,
}

impl egui_tetra::State<anyhow::Error> for Game {
    fn ui(&mut self, _ctx: &mut tetra::Context, egui_ctx: &egui::CtxRef) -> anyhow::Result<()> {
        match &self.ui_state {
            UIState::Inventory(vec) => {
                egui::Window::new("Inventory")
                    .fixed_pos(Pos2::new(1., 1.))
                    .show(egui_ctx, |ui| {
                        ui.label("Items in inventory");
                        for i in vec {
                            ui.label(i.as_str());
                        }
                    });
            }
            UIState::No => {}
            UIState::Debug => {
                egui::Window::new("Debug")
                    .fixed_pos(Pos2::new(1., 1.))
                    .show(egui_ctx, |_ui| {});
            }
        }
        Ok(())
    }
    fn draw(&mut self, ctx: &mut Context, _egui_ctx: &egui::CtxRef) -> anyhow::Result<()> {
        let (w, h) = window::get_size(ctx);
        self.scaler.set_outer_size(w, h);
        graphics::set_canvas(ctx, self.scaler.canvas());
        graphics::clear(ctx, Color::rgb(0., 0., 0.));
        RenderSystem.run(self, ctx).unwrap();
        graphics::reset_canvas(ctx);
        graphics::clear(ctx, Color::rgb(0., 0., 0.));
        self.scaler.draw(ctx);
        Ok(())
    }
    fn update(&mut self, ctx: &mut Context, _egui_ctx: &egui::CtxRef) -> anyhow::Result<()> {
        for system in self.game_systems.iter() {
            system.run(self, ctx)?
        }
        for system in self.world_systems.iter() {
            system.run(&self.world, ctx)?
        }
        Ok(())
    }
}

impl Game {
    fn new(ctx: &mut Context) -> tetra::Result<Game> {
        let exe_path = env::current_exe().expect("Ты ебанутый? Ты что там делаешь?");
        let assets_path = exe_path
            .parent()
            .and_then(|p| p.parent())
            .map(|p| p.join("assets"))
            .filter(|p| p.exists())
            .unwrap_or(
                env::current_dir()
                    .expect("Ты как сюда залез?")
                    .join("assets"),
            );
        let game_systems: GameSystems = init_systems![];
        let world_systems: WorldSystems =
            init_systems![MovePlayerSystem, FovComputeSystem, MemorySystem];
        let resources = Resources::load(ctx, &assets_path);
        let mut world = World::new();
        let map = WorldMap::new();
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
            game_systems,
            world_systems,
            ui_state: UIState::Debug,
        })
    }
}

pub fn main() -> anyhow::Result<()> {
    ContextBuilder::new("S", 500, 500)
        .quit_on_escape(true)
        .resizable(true)
        .show_mouse(true)
        .grab_mouse(false)
        //.timestep(tetra::time::Timestep::Fixed(20.))
        .build()?
        .run(|ctx| Ok(StateWrapper::new(Game::new(ctx)?)))
}
