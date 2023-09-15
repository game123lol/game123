use std::collections::HashMap;

use hecs::World;
use tetra::{
    graphics::{DrawParams, Texture},
    math::Vec2,
    Context,
};

use crate::{
    entities::{Item, Mob, Player, Position, Renderable, Sight},
    map::Map,
};

pub fn run_render_system_fov(
    world: &World,
    ctx: &mut Context,
    resources: &HashMap<String, Texture>,
    canvas_size: (i32, i32),
) {
    if let Some((_, (map,))) = world.query::<(&mut Map,)>().iter().next() {
        if let Some((_, (_, Position(cam_pos), Sight(sight_positions)))) =
            world.query::<(&Player, &Position, &Sight)>().iter().next()
        {
            let (w, h) = canvas_size;
            let mut renderable_mobs = world.query::<(&Renderable, &Position, &Mob)>();
            let renderable_mobs = renderable_mobs.iter().map(|(e, (r, p, _))| (e, (r, p)));
            let mut renderable_items = world.query::<(&Renderable, &Position, &Item)>();
            let renderable_items = renderable_items.iter().map(|(e, (r, p, _))| (e, (r, p)));

            //координаты предметов которые надо рендерить
            let mut ren_map: HashMap<(i32, i32), Vec<&Renderable>> = HashMap::new();

            // разгоняем по тайлам всё что нужно рендерить
            for (_, (renderable, Position(pos))) in renderable_items.chain(renderable_mobs) {
                if let Some(vec) = ren_map.get_mut(&(pos[0], pos[1])) {
                    vec.push(renderable);
                } else {
                    ren_map.insert((pos[0], pos[1]), vec![renderable]);
                }
            }
            let mut sight_positions = sight_positions
                .iter()
                .map(|&a| a)
                .collect::<Vec<(i32, i32)>>();
            sight_positions.sort_by(|&a, &b| a.1.partial_cmp(&b.1).unwrap());
            sight_positions.sort_by(|&a, &b| a.0.partial_cmp(&b.0).unwrap());
            for pos in sight_positions {
                let (x, y) = pos;
                let x_real = x + cam_pos.x;
                let y_real = y + cam_pos.y;
                let (ch_x, ch_y) = Map::xy_chunk(x_real, y_real);
                let chunk = map.get_chunk_or_create(ch_x, ch_y);
                let idx = Map::xy_index_chunk(x_real, y_real);
                let tile = &chunk.tiles[idx];
                let position = Vec2::new(w as f32 / 2., h as f32 / 2.)
                    + Vec2::new((7 * (x - y)) as f32, (4 * (y + x)) as f32);

                let params = DrawParams::new().position(position);
                let is_full = x <= 0 && 0 >= y || tile.partial_sprite.is_none();
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
                if let Some(renderables) = ren_map.get(&(x + cam_pos.x, y + cam_pos.y)) {
                    for Renderable(texture_name, texture_rect) in renderables {
                        resources.get(texture_name).unwrap().draw_region(
                            ctx,
                            *texture_rect,
                            params.clone(),
                        );
                    }
                }
            }
        }
    }
}
