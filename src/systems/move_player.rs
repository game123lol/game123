use hecs::World;
use tetra::{
    input::{get_keys_down, Key},
    math::Vec2,
    Context,
};

use crate::{
    components::{Player, Position},
    map::Map,
};

pub fn move_player_system(world: &mut World, ctx: &Context) {
    let mut binding = world.query::<(&Player, &mut Position)>();
    let (_, (_, Position(pos))) = binding.into_iter().next().expect(
        "Can't run move_player system without a entity with Player and Position components",
    );
    let mut binding = world.query::<(&mut Map,)>();
    let (_, (map,)) = binding
        .into_iter()
        .next()
        .expect("Can't run move_player system without a entity with Map component");
    for key in get_keys_down(&ctx) {
        let mut step = *pos;
        match key {
            Key::W => {
                step += Vec2::new(0, -1);
            }
            Key::S => {
                step += Vec2::new(0, 1);
            }
            Key::A => {
                step += Vec2::new(-1, 0);
            }
            Key::D => {
                step += Vec2::new(1, 0);
            }
            _ => {}
        }
        if !map.get_obstacle_or_create(step.x, step.y) {
            *pos = step;
        }
    }
}
