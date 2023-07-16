use std::{collections::HashMap, f32::consts::PI};

use hecs::World;
use tetra::{
    graphics::{DrawParams, Texture},
    math::{Mat4, Vec2},
    window, Context,
};

use crate::{map::Map, Player, Position, Renderable};

pub fn run_render_system(world: &World, ctx: &mut Context, resources: &HashMap<String, Texture>) {
    if let Some((_, (_, Position(cam_pos)))) = world.query::<(&Player, &Position)>().iter().next() {
        for (e, (Renderable(texture, rect), Position(pos))) in
            world.query::<(&Renderable, &Position)>().iter()
        {
            let (w, h) = window::get_size(ctx);
            let texture = resources.get(texture).unwrap();
            let position = Vec2::new(w as f32 / 2., h as f32 / 2.)
                + Vec2::new(
                    (7 * (pos.x - pos.y - cam_pos.x + cam_pos.y)) as f32,
                    (4 * (pos.y + pos.x - cam_pos.y - cam_pos.x)) as f32,
                );
            let params = DrawParams::new().position(position);
            texture.draw_region(ctx, *rect, params)
        }
    }
}

pub fn run_map_render_system_iso(
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
                let position = Vec2::new(w as f32 / 2., h as f32 / 2.)
                    + Vec2::new(
                        (7 * (x - y - cam_pos.x + cam_pos.y)) as f32,
                        (4 * (y + x - cam_pos.y - cam_pos.x)) as f32,
                    );
                let params = DrawParams::new().position(position);
                resources.get(&tile.texture_name).unwrap().draw_region(
                    ctx,
                    tile.texture_rect,
                    params,
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
