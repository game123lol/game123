use std::{
    collections::HashMap,
    sync::{Arc, MutexGuard},
};

use macroquad::{
    miniquad::window::screen_size,
    prelude::{Color, Vec2},
    texture::{draw_texture_ex, DrawTextureParams},
};

use crate::{
    components::Position,
    items::Item,
    map::{Chunk, Map, WorldMap},
    need_components,
    player::Player,
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

/// Компонент, используемый в функции рендера. Все сущности, обладающие этим компонентом,
/// а так же компонентами Position и Item или Mob, будут отрисованы.
/// Компонент содержит в себе название спрайта, который будет отрисован.
/// По этому названию будет сделан запрос в хранилище спрайтов resources (поле Game).
#[derive(Debug)]
pub struct Renderable(pub Arc<str>);

pub fn run_render_system(game: &mut Game) -> super::Result {
    let world = &game.world;
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
    let (w, h) = screen_size();
    let mut renderable_mobs = world.query::<(&Renderable, &Position, &Mob)>();
    let renderable_mobs = renderable_mobs.iter().map(|(e, (r, p, _))| (e, (r, p)));
    let mut renderable_items = world.query::<(&Renderable, &Position, &Item)>();
    let renderable_items = renderable_items.iter().map(|(e, (r, p, _))| (e, (r, p)));

    //координаты предметов которые надо рендерить
    let mut ren_map: HashMap<(i32, i32, i32), Vec<&Renderable>> = HashMap::new();

    // разгоняем по тайлам всё что нужно рендерить
    for (_, (renderable, Position(pos))) in renderable_items.chain(renderable_mobs) {
        ren_map
            .entry((pos.x, pos.y, pos.z))
            .and_modify(|e| e.push(renderable))
            .or_insert(vec![renderable]);
    }
    let render_radius = *sight_radius as i32 + 5;

    let mut prev_chunk_mutex: Option<(MutexGuard<Chunk>, i32, i32, i32)> = None;

    let base_color = Color::from_hex(0xFFFFFF);
    let shadowed_color = Color::from_hex(0x555555);
    for z in -render_radius..=render_radius {
        for y in -render_radius..=render_radius {
            for x in -render_radius..=render_radius {
                if (x.pow(2) + y.pow(2) + z.pow(2)) > (1 + render_radius).pow(2) {
                    continue;
                }
                let position = Vec2::new(w / 2., h / 2.)
                    + Vec2::new(
                        game.scale * (16 * (x - y)) as f32,
                        game.scale * (7 * (y + x)) as f32,
                    )
                    + Vec2::new(0., 0. - 15. * game.scale * z as f32);
                if position.x < -30. * game.scale
                    || position.x > w
                    || position.y < -30. * game.scale
                    || position.y > h
                {
                    continue;
                }
                let x_real = x + cam_pos.x;
                let y_real = y + cam_pos.y;
                let z_real = z + cam_pos.z;
                let (ch_x, ch_y, ch_z) = WorldMap::xy_chunk(x_real, y_real, z_real);
                let chunk = match prev_chunk_mutex {
                    Some((ref mutex, p_ch_x, p_ch_y, p_ch_z))
                        if p_ch_x == ch_x && p_ch_y == ch_y && p_ch_z == ch_z =>
                    {
                        mutex
                    }
                    _ => {
                        let new_chunk_mutex = map.get_chunk(ch_x, ch_y, ch_z);
                        if new_chunk_mutex.is_none() {
                            continue;
                        }
                        let new_chunk_mutex = new_chunk_mutex.unwrap().lock().unwrap();
                        prev_chunk_mutex = Some((new_chunk_mutex, ch_x, ch_y, ch_z));
                        &prev_chunk_mutex.as_ref().unwrap().0
                    }
                };

                let memory_chunk = map_memory.get_chunk(ch_x, ch_y, ch_z);
                let tile = chunk.get_tile(x_real, y_real, z_real);
                let sprite = resources
                    .assets
                    .sprites
                    .get(tile.full_sprite)
                    .unwrap_or_else(|| sprite_not_found(tile.full_sprite));

                let is_visible = sight_positions.contains(&(x, y, z));
                let is_memorized = memory_chunk.map_or(false, |a| {
                    a.lock().unwrap().is_memorized(x_real, y_real, z_real)
                });

                if !is_visible && !is_memorized {
                    continue;
                }

                let color = if !is_visible && is_memorized {
                    shadowed_color
                } else {
                    base_color
                };

                if tile.name != "empty" {
                    let params = DrawTextureParams {
                        source: Some(sprite.rect),
                        dest_size: Some(Vec2::new(
                            sprite.rect.w * game.scale,
                            sprite.rect.h * game.scale,
                        )),
                        ..Default::default()
                    };
                    draw_texture_ex(&sprite.texture, position.x, position.y, color, params);
                }

                if let Some(renderables) = ren_map.get(&(x_real, y_real, z_real)) {
                    for Renderable(name) in renderables {
                        let sprite = resources
                            .assets
                            .sprites
                            .get(name)
                            .unwrap_or_else(|| sprite_not_found(name));
                        let shift_x = (30. - sprite.rect.w) / 2. * game.scale;
                        let shift_y = (35. - sprite.rect.h) / 2. * game.scale;
                        let params = DrawTextureParams {
                            source: Some(sprite.rect),
                            dest_size: Some(Vec2::new(
                                sprite.rect.w * game.scale,
                                sprite.rect.h * game.scale,
                            )),
                            ..Default::default()
                        };
                        draw_texture_ex(
                            &sprite.texture,
                            position.x + shift_x,
                            position.y + shift_y,
                            Color::from_hex(0xFFFFFF),
                            params,
                        );
                    }
                }
            }
        }
    }
    Ok(())
}
