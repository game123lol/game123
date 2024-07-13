mod components;
mod items;
mod map;
mod mob;
mod player;
mod resources;
mod systems;
mod tests;
mod ui;
use components::Position;

use hecs::{CommandBuffer, World};
use items::Item;
use macroquad::{
    prelude::Color,
    window::{clear_background, next_frame, Conf},
};
use map::WorldMap;
use mob::{Inventory, Log, Mob};
use player::{get_player_items, new_player, Player};
use resources::Resources;
use std::{collections::HashMap, env, sync::Mutex, time::Duration};
use systems::{
    health::{Body, BodyPart, BodyPartPart, Organ, Wound},
    movement::{dir_to_vec3, WantsMove},
    render::run_render_system,
    GameSystem, WorldSystem,
};

use ui::{set_skin, UIConfig, UIState};

use crate::systems::health::WantsAttack;

type GameSystems = Vec<GameSystem>;
type WorldSystems = Vec<WorldSystem>;

type Type = World;

type GameHasher = fxhash::FxBuildHasher;

pub fn hasher() -> GameHasher {
    fxhash::FxBuildHasher::default()
}

#[derive(Clone)]
pub enum Property {
    Int(i32),
    String(String),
    Float(f64),
    Marker,
}

pub struct Game {
    world: Type,
    resources: Resources,
    game_systems: GameSystems,
    world_systems: WorldSystems,
    ui: UIState,
    ui_config: UIConfig,
    next_action: PlayerAction,
    is_paused: bool,
    is_needed_redraw: Mutex<bool>,
    scale: f32,
    statistics: Mutex<Statistics>,
}

#[derive(Clone)]
pub struct Statistics {
    systems_average: HashMap<String, (Duration, u32), GameHasher>,
}

impl Statistics {
    pub fn new() -> Self {
        Self {
            systems_average: HashMap::with_hasher(hasher()),
        }
    }
    pub fn show(&self) -> String {
        let mut total = Duration::default();
        let mut result = String::new();
        for (name, (stat, _)) in &self.systems_average {
            total += *stat;
            result.push_str(format!("{name} system elapsed: {stat:2?}\n").as_str());
        }
        result.push_str(format!("total systems elapsed: {total:2?}\n").as_str());
        result
    }
    pub fn update_stat(&mut self, time: Duration, system: String) {
        self.systems_average
            .entry(system)
            .and_modify(|(avg, counter)| {
                *counter += 1;
                *avg = ((*counter - 1) * *avg + time) / *counter;
                if *counter > 100 {
                    *counter = 1;
                    *avg = time;
                }
            })
            .or_insert((time, 1));
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum PlayerAction {
    Move(Direction),
    OpenInventory,
    CloseInventory,
    OpenLog,
    CloseLog,
    PickUpItem,
    Nothing,
    Zoom,
    Unzoom,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Direction {
    Forward,
    Back,
    Left,
    Right,
    Up,
    Down,
}

enum Action {
    PlayerAction(PlayerAction),
    UIEvent {},
}

impl Game {
    async fn new() -> anyhow::Result<Game> {
        set_skin().await;
        let exe_path = env::current_exe().expect("Ты ебанутый? Ты что там делаешь?");
        let data_path = exe_path
            .parent()
            .and_then(|p| p.parent())
            .map(|p| p.join("data"))
            .filter(|p| p.exists())
            .unwrap_or(env::current_dir().expect("Ты как сюда залез?").join("data"));
        let mut resources = Resources::load(&data_path).await;
        let game_systems: GameSystems = vec![GameSystem::InputSystem];
        let world_systems: WorldSystems = vec![
            WorldSystem::Move,
            WorldSystem::FovCompute,
            WorldSystem::Memory,
            WorldSystem::Pathfinding,
            WorldSystem::Attack,
        ];
        let mut world = World::new();
        let map = WorldMap::new();
        world.spawn((map,));
        let mut player = new_player();
        let body = Body::new().with_part(
            "head".into(),
            BodyPart::new().with_part(
                "head".into(),
                BodyPartPart::new().with_organ("eyes".into(), Organ::new()),
            ),
        );
        player.add(body);
        world.spawn(player.build());
        let mut item = Item::new("thing1".into(), "item".into());
        item.add_props(&[("huy".into(), Property::Marker)]);
        world.spawn(item.to_map_entity(2, 2, 0));
        let mut item = Item::new("thing2".into(), "item".into());
        item.add_props(&[("huy".into(), Property::Marker)]);
        world.spawn(item.to_map_entity(2, 3, 0));
        let mut item = Item::new("thing3".into(), "item".into());
        item.add_props(&[("huy".into(), Property::Marker)]);
        world.spawn(item.to_map_entity(2, 4, 0));

        let builder = resources.entity_templates.get_mut("nettle").unwrap();
        world.spawn(builder.build());

        Ok(Game {
            world,
            resources,
            game_systems,
            world_systems,
            ui: UIState::Debug,
            ui_config: UIConfig::default(),
            next_action: PlayerAction::Nothing,
            is_paused: false,
            is_needed_redraw: Mutex::new(true),
            scale: 1.,
            statistics: Mutex::new(Statistics::new()),
        })
    }

    async fn draw(&self) -> anyhow::Result<()> {
        let mut is_needed_redraw = self.is_needed_redraw.lock().unwrap();
        if *is_needed_redraw || self.is_paused {
            clear_background(Color::from_hex(0x000000));
            let now = std::time::Instant::now();
            run_render_system(self)?;
            let elapsed = now.elapsed();
            let mut stats = self.statistics.lock().unwrap();
            stats.update_stat(elapsed, "Render system".into());
            *is_needed_redraw = false;
        }

        Ok(())
    }

    async fn draw_ui(&self) -> anyhow::Result<()> {
        match self.ui {
            UIState::No => {}
            UIState::Inventory { ref items } => ui::inventory(items),
            UIState::Log { ref text } => ui::log(text),
            UIState::Debug => ui::debug(&self.statistics.lock().unwrap().to_owned()),
        }
        Ok(())
    }

    async fn update(&mut self) -> anyhow::Result<()> {
        if self.is_paused {
            let mut is_needed_redraw = self.is_needed_redraw.lock().unwrap();
            match self.next_action {
                PlayerAction::Move(dir) => {
                    let mut bind = self.world.query::<(&Player, &Position)>();
                    let (e, (_, Position(pos))) = bind
                        .into_iter()
                        .next()
                        .expect("Персонаж потерялся. Как так?");
                    let pos = *pos;
                    drop(bind);
                    let mut mobs = self.world.query::<(&Mob, &Position)>();
                    let mut cmd = CommandBuffer::new();
                    if let Some((target, _)) = mobs
                        .iter()
                        .find(|(_, (_, Position(mob_pos)))| *mob_pos == pos + dir_to_vec3(&dir))
                    {
                        cmd.insert(e, (WantsAttack(Wound::Bruised, target),));
                    } else {
                        cmd.insert(e, (WantsMove(dir),));
                    }
                    drop(mobs);
                    cmd.run_on(&mut self.world);
                    self.is_paused = false;
                    *is_needed_redraw = true;
                }
                PlayerAction::OpenInventory => {
                    self.ui = UIState::Inventory {
                        items: get_player_items(&self.world)?,
                    }
                }
                PlayerAction::PickUpItem => {
                    let mut bind_player = self
                        .world
                        .query::<(&Player, &Position, &mut Inventory, &mut Log)>();
                    let (_, (_, player_pos, inventory, log)) = bind_player
                        .into_iter()
                        .next()
                        .expect("Персонаж потерялся. Как так?");
                    let mut bind_item = self.world.query::<(&Item, &Position)>();
                    let items = bind_item.into_iter();
                    let mut cmd = CommandBuffer::new();
                    for (e, (item, pos)) in items {
                        if *pos == *player_pos {
                            log.write(&format!("Picked up {}", item.name.clone()));
                            inventory.0.push(item.clone());
                            cmd.despawn(e);
                            break;
                        }
                    }
                    drop(bind_item);
                    drop(bind_player);
                    cmd.run_on(&mut self.world);
                    self.is_paused = false;
                    *is_needed_redraw = true;
                }
                PlayerAction::OpenLog => {
                    let mut bind_player = self.world.query::<(&Player, &Log)>();
                    let (_, (_, log)) = bind_player
                        .into_iter()
                        .next()
                        .expect("Персонаж потерялся. Как так?");
                    self.ui = UIState::Log {
                        text: log.0.clone(),
                    }
                }
                PlayerAction::CloseLog | PlayerAction::CloseInventory => {
                    self.ui = UIState::No;
                }
                PlayerAction::Zoom => {
                    self.scale += 0.1;
                }
                PlayerAction::Unzoom => {
                    self.scale -= 0.1;
                }
                _ => {}
            }
            drop(is_needed_redraw);
            for system in self.game_systems.clone().iter() {
                system.run(self)?
            }
        } else {
            for system in self.world_systems.iter() {
                let now = std::time::Instant::now();
                system.run(&mut self.world)?;
                let elapsed = now.elapsed();
                let mut stats = self.statistics.lock().unwrap();
                stats.update_stat(elapsed, format!("{system:?}"));
            }

            self.is_paused = true;
        }
        Ok(())
    }
}

fn window_conf() -> Conf {
    use macroquad::miniquad;
    Conf {
        window_title: "Window Conf".to_owned(),
        fullscreen: false,
        platform: miniquad::conf::Platform {
            linux_backend: miniquad::conf::LinuxBackend::X11Only,
            ..Default::default()
        },
        ..Default::default()
    }
}

#[macroquad::main(window_conf)]
async fn main() -> anyhow::Result<()> {
    let mut game = Game::new().await?;
    loop {
        let now = std::time::Instant::now();
        game.update().await?;
        game.draw().await?;
        game.draw_ui().await?;
        next_frame().await;
        let _elapsed = now.elapsed();
        let fps = (1. / now.elapsed().as_secs_f64()) as i32;
        dbg!(fps);
    }
}
