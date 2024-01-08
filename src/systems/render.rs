use std::{collections::BTreeMap, sync::Mutex};

use tetra::{
    graphics::{Color, DrawParams},
    math::Vec2,
    Context,
};

use crate::{
    components::{Item, MapMemory, Mob, Player, Position, Renderable, Sight},
    map::{Chunk, Map, WorldMap},
    need_components, Game,
};

//TODO: Сделать ошибки об отсутствии спрайтов более информативными
fn sprite_not_found<T>(name: &str) -> T {
    panic!(
        "Ты какой-то неправильный спрайт ({}) дёргаешь переписывай",
        name
    );
}

const fn xy_tile(num: u32, render_radius: u32) -> (i32, i32) {
    (
        -(render_radius as i32) + (num % (render_radius * 2)) as i32,
        -(render_radius as i32) + (num / (render_radius * 2)) as i32,
    )
}

pub fn run_render_system(game: &mut Game, ctx: &mut Context) -> super::Result {
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
        ren_map
            .entry((pos[0], pos[1]))
            .and_modify(|e| e.push(renderable))
            .or_insert(vec![renderable]);
    }
    let render_radius = sight_radius + 30;

    let render_coords = (0..render_radius * render_radius * 4)
        .map(|n| xy_tile(n, render_radius))
        .filter(move |(x, y)| ((x * x + y * y) as f64).sqrt() < render_radius as f64);

    let mut chunk_cache: Vec<((i32, i32), Option<&Mutex<Chunk>>)> = Vec::new();

    for (x, y) in render_coords {
        let position = Vec2::new(w as f32 / 2., h as f32 / 2.)
            + Vec2::new((14 * (x - y)) as f32, (7 * (y + x)) as f32);
        if position.x < -20. || position.x > w as f32 || position.y < -20. || position.y > h as f32
        {
            continue;
        }
        let x_real = x + cam_pos.x;
        let y_real = y + cam_pos.y;
        let (ch_x, ch_y) = WorldMap::xy_chunk(x_real, y_real);
        let chunk = if let Some((_, chunk)) = chunk_cache.iter().rev().find(|a| a.0 == (ch_x, ch_y))
        {
            *chunk
        } else {
            let link = map.get_chunk(ch_x, ch_y);
            chunk_cache.push(((ch_x, ch_y), link));
            link
        };

        if chunk.is_none() {
            continue;
        }
        let chunk = chunk.unwrap().lock().unwrap();
        let memory_chunk = map_memory.get_chunk(ch_x, ch_y);
        let idx = WorldMap::xy_index_chunk(x_real, y_real);
        let tile = &chunk.tiles[idx];
        let is_full = x <= 0 && 0 >= y || !chunk.obstacles[idx];
        let sprite = resources
            .sprites
            .get(tile.full_sprite)
            .unwrap_or_else(|| sprite_not_found(tile.full_sprite));
        let fallback_sprite = tile.fallback_sprite.map(|s| {
            resources
                .sprites
                .get(s)
                .unwrap_or_else(|| sprite_not_found(s))
        });

        let is_visible = sight_positions.contains(&(x, y));
        let is_memorized = memory_chunk.map_or(false, |a| a.lock().unwrap().memorized[idx]);

        if !is_visible && !is_memorized {
            continue;
        }

        let mut params = DrawParams::new().position(position);

        if !is_visible && is_memorized {
            params = params.color(Color::rgb8(128, 128, 128))
        }

        if !is_full && is_visible {
            params.color = params.color.with_alpha(0.7);
        }

        if let Some(fallback_sprite) = fallback_sprite {
            fallback_sprite
                .texture
                .draw_region(ctx, fallback_sprite.rect, params.clone());
        }
        sprite.texture.draw_region(ctx, sprite.rect, params.clone());

        if let Some(renderables) = ren_map.get(&(x_real, y_real)) {
            for Renderable(name) in renderables {
                let sprite = resources
                    .sprites
                    .get(*name)
                    .unwrap_or_else(|| sprite_not_found(name));
                let shift_x = (30. - sprite.rect.width) / 2.;
                let shift_y = (40. - sprite.rect.height) / 2.;
                let renderable_params = params
                    .clone()
                    .position(params.position + Vec2::new(shift_x, shift_y));
                sprite
                    .texture
                    .draw_region(ctx, sprite.rect, renderable_params);
            }
        }
    }
    Ok(())
}
