use std::collections::BTreeMap;

use tetra::{
    graphics::{Color, DrawParams},
    math::Vec2,
    Context,
};

use crate::{
    components::{Item, MapMemory, Mob, Player, Position, Renderable, Sight},
    map::{Map, WorldMap},
    need_components,
};

use super::GameSystem;

//TODO: Сделать ошибки об отсутствии спрайтов более информативными

pub struct RenderSystem;

impl GameSystem for RenderSystem {
    fn run(&self, game: &crate::Game, ctx: &mut Context) -> super::Result {
        let world = &game.world;
        let canvas_size = game.scaler.inner_size();
        let resources = &game.resources;
        let mut query = world.query::<(&WorldMap,)>();
        let (_, (map,)) = query
            .iter()
            .next()
            .ok_or(need_components!(RenderSystem, Map))?;

        let mut query = world.query::<(&Player, &Position, &Sight, &MapMemory)>();
        let (_, (_, Position(cam_pos), Sight(sight_radius, sight_positions), map_memory)) =
            query.iter().next().ok_or(need_components!(
                RenderSystem,
                Player,
                Position,
                Sight,
                MapMemory
            ))?;
        let (w, h) = canvas_size;
        let mut renderable_mobs = world.query::<(&Renderable, &Position, &Mob)>();
        let renderable_mobs = renderable_mobs.iter().map(|(e, (r, p, _))| (e, (r, p)));
        let mut renderable_items = world.query::<(&Renderable, &Position, &Item)>();
        let renderable_items = renderable_items.iter().map(|(e, (r, p, _))| (e, (r, p)));

        //координаты предметов которые надо рендерить
        let mut ren_map: BTreeMap<(i32, i32), Vec<&Renderable>> = BTreeMap::new();

        // разгоняем по тайлам всё что нужно рендерить
        for (_, (renderable, Position(pos))) in renderable_items.chain(renderable_mobs) {
            if let Some(vec) = ren_map.get_mut(&(pos[0], pos[1])) {
                vec.push(renderable);
            } else {
                ren_map.insert((pos[0], pos[1]), vec![renderable]);
            }
        }
        let render_radius = sight_radius + 30;
        for pos_y in -(render_radius as i32)..render_radius as i32 {
            'x_row: for pos_x in -(render_radius as i32)..render_radius as i32 {
                let (x, y) = (pos_x, pos_y);
                let x_real = x + cam_pos.x;
                let y_real = y + cam_pos.y;
                let (ch_x, ch_y) = WorldMap::xy_chunk(x_real, y_real);
                let chunk = map.get_chunk(ch_x, ch_y);
                if chunk.is_none() {
                    continue 'x_row;
                }
                let chunk = chunk.unwrap();
                let memory_chunk = map_memory.get_chunk(ch_x, ch_y);
                let idx = WorldMap::xy_index_chunk(x_real, y_real);
                let tile = &chunk.tiles[idx];
                let position = Vec2::new(w as f32 / 2., h as f32 / 2.)
                    + Vec2::new((7 * (x - y)) as f32, (4 * (y + x)) as f32);

                let params = DrawParams::new().position(position);
                let is_full = x <= 0 && 0 >= y || tile.partial_sprite.is_none();
                let sprite = if is_full {
                    &tile.full_sprite
                } else {
                    tile.partial_sprite.as_ref().unwrap()
                };
                let sprite = resources.sprites.get(sprite).unwrap();
                let in_render_radius =
                    ((pos_x * pos_x + pos_y * pos_y) as f64).sqrt() < render_radius as f64;
                let is_visible = sight_positions.contains(&(pos_x, pos_y));
                if is_visible {
                    sprite.texture.draw_region(ctx, sprite.rect, params.clone());
                } else if in_render_radius
                    && memory_chunk.is_some()
                    && memory_chunk.unwrap().memorized[idx]
                {
                    sprite.texture.draw_region(
                        ctx,
                        sprite.rect,
                        params.clone().color(Color::rgb8(128, 128, 128)),
                    );
                }

                if let Some(renderables) = ren_map.get(&(x + cam_pos.x, y + cam_pos.y)) {
                    for Renderable(name) in renderables {
                        let sprite = resources.sprites.get(name).unwrap();
                        if is_visible {
                            sprite.texture.draw_region(ctx, sprite.rect, params.clone());
                        } else if in_render_radius
                            && memory_chunk.is_some()
                            && memory_chunk.unwrap().memorized[idx]
                        {
                            sprite.texture.draw_region(
                                ctx,
                                sprite.rect,
                                params.clone().color(Color::rgb8(128, 128, 128)),
                            );
                        }
                    }
                }
            }
        }
        Ok(())
    }
}
