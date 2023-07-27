use std::collections::HashMap;

use hecs::World;
use tetra::{
    graphics::{DrawParams, Texture},
    math::Vec2,
    Context,
};

use crate::{
    entities::{Item, Mob, Player, Position, Renderable},
    map::{Map, Sprite},
};

pub fn run_render_system(
    world: &World,
    ctx: &mut Context,
    resources: &HashMap<String, Texture>,
    canvas_size: (i32, i32),
) {
    if let Some((_, (map,))) = world.query::<(&Map,)>().iter().next() {
        if let Some((_, (_, Position(cam_pos)))) =
            world.query::<(&Player, &Position)>().iter().next()
        {
            let (mut x, mut y) = (0, 0);
            let (w, h) = canvas_size;
            let mut renderable_mobs = world.query::<(&Renderable, &Position, &Mob)>();
            let renderable_mobs = renderable_mobs.iter().map(|(e, (r, p, _))| (e, (r, p)));
            let mut renderable_items = world.query::<(&Renderable, &Position, &Item)>();
            let renderable_items = renderable_items.iter().map(|(e, (r, p, _))| (e, (r, p)));
            let mut ren_map: HashMap<(i32, i32), Vec<&Renderable>> = HashMap::new();

            // разгоняем по тайлам всё что нужно рендерить
            for (_, (renderable, Position(pos))) in renderable_items.chain(renderable_mobs) {
                if let Some(vec) = ren_map.get_mut(&(pos[0], pos[1])) {
                    vec.push(renderable);
                } else {
                    ren_map.insert((pos[0], pos[1]), vec![renderable]);
                }
            }

            for tile in &map.tiles {
                let position = Vec2::new(w as f32 / 2., h as f32 / 2.)
                    + Vec2::new(
                        (7 * (x - y - cam_pos.x + cam_pos.y)) as f32,
                        (4 * (y + x - cam_pos.y - cam_pos.x)) as f32,
                    );
                let params = DrawParams::new().position(position);
                let is_full = cam_pos.x > x || cam_pos.y > y || tile.partial_sprite.is_none();
                let sprite = if is_full {
                    &tile.full_sprite
                } else {
                    &tile.partial_sprite.as_ref().unwrap()
                };
                resources.get(&sprite.src_name).unwrap().draw_region(
                    ctx,
                    sprite.rect,
                    params.clone(),
                );

                if let Some(renderables) = ren_map.get(&(x, y)) {
                    for Renderable(texture_name, texture_rect) in renderables {
                        resources.get(texture_name).unwrap().draw_region(
                            ctx,
                            *texture_rect,
                            params.clone(),
                        );
                    }
                }

                x += 1;
                if x >= map.size.0 as i32 {
                    x = 0;
                    y += 1;
                }
            }
        }
    }
}
