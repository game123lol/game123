use std::{
    collections::BTreeMap,
    sync::{Arc, Mutex},
};

use tetra::{
    graphics::{mesh::Mesh, Color, DrawParams},
    math::Vec2,
    Context,
};

use crate::{
    components::Position,
    items::Item,
    map::{Chunk, Map, WorldMap},
    need_components,
    player::Player,
    resources::Sprite,
    Game, Mob,
};

use super::{fov_compute::Sight, memory::MapMemory};

//TODO: Сделать ошибки об отсутствии спрайтов более информативными
fn sprite_not_found<T>(name: &str) -> T {
    panic!(
        "Ты какой-то неправильный спрайт ({}) дёргаешь переписывай",
        name
    );
}

/// отображение из множества целых чисел в упорядоченное множество координат тайлов
const fn xy_tile(num: u32, render_radius: u32) -> (i32, i32, i32) {
    let num = num as i32;
    let render_radius = render_radius as i32;
    let render_dyameter = render_radius * 2 + 1;
    let z_level = -(render_radius) + (num / (render_dyameter * render_dyameter));
    let z_offset = (render_radius + z_level) * (render_dyameter * render_dyameter);
    let res = (
        -(render_radius) + ((num - z_offset) % render_dyameter),
        -(render_radius) + ((num - z_offset) / render_dyameter),
        z_level,
    );
    res
}

/// Компонент, используемый в функции рендера. Все сущности, обладающие этим компонентом,
/// а так же компонентами Position и Item или Mob, будут отрисованы.
/// Компонент содержит в себе название спрайта, который будет отрисован.
/// По этому названию будет сделан запрос в хранилище спрайтов resources (поле Game).
#[derive(Debug)]
pub struct Renderable(pub Arc<str>);

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
    let mut ren_map: BTreeMap<(i32, i32, i32), Vec<&Renderable>> = BTreeMap::new();

    // разгоняем по тайлам всё что нужно рендерить
    for (_, (renderable, Position(pos))) in renderable_items.chain(renderable_mobs) {
        ren_map
            .entry((pos.x, pos.y, pos.z))
            .and_modify(|e| e.push(renderable))
            .or_insert(vec![renderable]);
    }
    let render_radius = *sight_radius; // + 30;

    let render_dyameter = render_radius * 2 + 1;
    let render_coords = (0..render_dyameter * render_dyameter * render_dyameter)
        .map(|n| xy_tile(n, render_radius))
        .filter(move |(x, y, z)| {
            ((x * x + y * y + z * z) as f64).sqrt() < 1. + render_radius as f64
        }); //плоское ради совместимости

    // let mut chunk_cache: Vec<((i32, i32), Option<&Mutex<Chunk>>)> = Vec::new();

    for (x, y, z) in render_coords {
        let position = Vec2::new(w as f32 / 2., h as f32 / 2.) //центр экрана
            + Vec2::new((14 * (x - y)) as f32, (7 * (y + x)) as f32)
            + Vec2::new(0., 0. - 25. * z as f32);
        if position.x < -20. || position.x > w as f32 || position.y < -20. || position.y > h as f32
        {
            continue;
        }
        let x_real = x + cam_pos.x;
        let y_real = y + cam_pos.y;
        let z_real = z + cam_pos.z;
        let (ch_x, ch_y, ch_z) = WorldMap::xy_chunk(x_real, y_real, z_real);
        // let chunk = if let Some((_, chunk)) = chunk_cache.iter().rev().find(|a| a.0 == (ch_x, ch_y))
        // {
        //     *chunk
        // } else {
        //     let link = map.get_chunk(ch_x, ch_y, ch_z);
        //     chunk_cache.push(((ch_x, ch_y), link));
        //     link
        // };
        let chunk = map.get_chunk(ch_x, ch_y, ch_z);

        if chunk.is_none() {
            continue;
        }
        let chunk = chunk.unwrap().lock().unwrap();
        let memory_chunk = map_memory.get_chunk(ch_x, ch_y, ch_z);
        let idx = WorldMap::xy_index_chunk(x_real, y_real, z_real);
        // let is_border = idx < 15 || idx % 15 == 0;
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

        let is_visible = sight_positions.contains(&(x, y, z));
        let is_memorized = memory_chunk.map_or(false, |a| a.lock().unwrap().memorized[idx]);

        if
        // !is_visible &&
        !is_memorized {
            continue;
        }

        let mut params = DrawParams::new().position(position);

        if !is_visible && is_memorized {
            params = params.color(Color::rgb8(128, 128, 128))
        }

        if !is_full && is_visible {
            params.color = params.color.with_alpha(0.7);
        }

        // if is_border {
        //     params.color = params.color.with_green(0.1).with_blue(0.1);
        // }

        let z_color = Color::rgb(
            (100.545 * z as f32).abs() % 1.,
            (100.235 * z as f32).abs() % 1.,
            (100.345 * z as f32).abs() % 1.,
        );
        if chunk.obstacles[idx] {
            params.color = z_color.with_alpha(1.);
        } else {
            params.color = z_color.with_alpha(0.1);
        }

        if let Some(fallback_sprite) = fallback_sprite {
            fallback_sprite
                .texture
                .draw_region(ctx, fallback_sprite.rect, params.clone());
        }
        sprite.texture.draw_region(ctx, sprite.rect, params.clone());

        params.color = Color::WHITE;

        if let Some(renderables) = ren_map.get(&(x_real, y_real, z_real)) {
            for Renderable(name) in renderables {
                let sprite = resources
                    .sprites
                    .get(name)
                    .unwrap_or_else(|| sprite_not_found(name));
                let shift_x = (30. - sprite.rect.width) / 2.;
                let shift_y = (45. - sprite.rect.height) / 2.;
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
