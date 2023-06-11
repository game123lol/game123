use std::collections::HashMap;

use hecs::World;
use tetra::{graphics::Texture, math::Vec2, window, Context};

use crate::{map::Map, Player, Position, Renderable};

pub fn run_render_system(world: &World, ctx: &mut Context, resources: &HashMap<String, Texture>) {
    if let Some((_, (_, Position(cam_pos)))) = world.query::<(&Player, &Position)>().iter().next() {
        for (e, (Renderable(texture, rect), Position(pos))) in
            world.query::<(&Renderable, &Position)>().iter()
        {
            let (w, h) = window::get_size(ctx);
            resources.get(texture).unwrap().draw_region(
                ctx,
                *rect,
                Vec2::new(
                    ((pos.x - cam_pos.x) * 16) as f32,
                    ((pos.y - cam_pos.y) * 16) as f32,
                ) + Vec2::new(w as f32 / 2., h as f32 / 2.),
            )
        }
    }
}

pub fn run_map_render_system(
    world: &World,
    ctx: &mut Context,
    resources: &HashMap<String, Texture>,
) {
    if let Some((e, (map,))) = world.query::<(&Map,)>().iter().next() {
        if let Some((_, (_, Position(cam_pos)))) =
            world.query::<(&Player, &Position)>().iter().next()
        {
            let (mut x, mut y) = (0, 0);
            for tile in &map.tiles {
                let (w, h) = window::get_size(ctx);
                resources.get(&tile.texture_name).unwrap().draw_region(
                    ctx,
                    tile.texture_rect,
                    Vec2::new(w as f32 / 2., h as f32 / 2.)
                        + Vec2::new((16 * (x - cam_pos.x)) as f32, (16 * (y - cam_pos.y)) as f32),
                );
                x += 1;
                if x >= map.size.0 as i32 {
                    x = 0;
                    y += 1;
                }
            }
        }
    }
}
