use std::{collections::BTreeSet, path::Components, sync::Mutex};

use hecs::World;
use tetra::math::{Vec2, Vec3};

use crate::{
    components::Position,
    map::{Chunk, Map, WorldMap},
    need_components,
    player::Player,
};

use rationals::ConstRational;

type Quad<T> = (T, T, T, T); //x1, y1, x2, y2

#[derive(Clone, Debug)]
struct Rect {
    depth: i32,
    slope: Slope<ConstRational>,
}

#[derive(Clone, Debug, Copy)]
struct Slope<T> {
    x1: T,
    y1: T,
    x2: T,
    y2: T,
}

/// Компонент, означающий, что сущность с этим компонентом имеет поле зрения.
/// Он имеет в себе радиус поля зрения и множество координат, которые сущность видит.
pub struct Sight(pub u32, pub BTreeSet<(i32, i32, i32)>);

enum Direction {
    Up,
    Down,
    Right,
    Left,
    Forward,
    Back,
}

pub fn run_fov_compute_system(world: &World) -> super::Result {
    let mut query = world.query::<(&mut WorldMap,)>();
    let (_, (map,)) = query
        .iter()
        .next()
        .ok_or(need_components!(FovSystem, WorldMap))?;
    let mut query = world.query::<(&Player, &Position, &mut Sight)>();
    let (_, (_, Position(cam_pos), Sight(sight_radius, sight_tiles))) = query
        .iter()
        .next()
        .ok_or(need_components!(FovComputeSystem, Player, Position, Sight))?;
    sight_tiles.clear();
    sight_tiles.insert((0, 0, 0));

    let dirs = [
        Direction::Up,
        Direction::Left,
        Direction::Down,
        Direction::Right,
        Direction::Forward,
        Direction::Back,
    ];
    let chunks_depth = (*sight_radius / 15 + 3 + 10) as i32;
    let current_chunk = WorldMap::xy_chunk(cam_pos.x, cam_pos.y, cam_pos.z);
    for i in -chunks_depth..=chunks_depth {
        for j in -chunks_depth..=chunks_depth {
            for k in -chunks_depth..=chunks_depth {
                map.get_chunk_or_create(
                    current_chunk.0 + i,
                    current_chunk.1 + j,
                    current_chunk.2 + k,
                );
            }
        }
    }
    sight_tiles.extend(
        dirs.iter()
            .flat_map(|dir| cast(cam_pos, dir, map, *sight_radius))
            .collect::<Vec<(i32, i32, i32)>>(),
    );
    Ok(())
}

impl Rect {
    const fn new(depth: i32, slope: Slope<ConstRational>) -> Self {
        Rect { depth, slope }
    }

    const fn tiles(&self) -> (i32, Quad<i32>) {
        let x1 = self.slope.x1.mul(ConstRational::new(self.depth, 1)).floor();
        let y1 = self.slope.y1.mul(ConstRational::new(self.depth, 1)).floor();
        let x2 = self
            .slope
            .x2
            .mul(ConstRational::new(self.depth, 1).add(ConstRational::new(1, 2)))
            .floor();
        let y2 = self
            .slope
            .y2
            .mul(ConstRational::new(self.depth, 1).add(ConstRational::new(1, 2)))
            .floor();

        (self.depth, (x1, y1, x2, y2))
    }
    const fn next(&self) -> Self {
        Rect::new(self.depth + 1, self.slope)
    }
}

pub const fn slope(depth: i32, x: i32, y: i32) -> (ConstRational, ConstRational) {
    let x_slope = ConstRational::new(2 * x - 1, 2 * depth);
    let y_slope = ConstRational::new(2 * y - 1, 2 * depth);
    (x_slope, y_slope)
}

const fn is_symmetric(rect: &Rect, x: i32, y: i32) -> bool {
    let x_symmetric = ConstRational::new(x, 1)
        .ge(ConstRational::new(rect.depth, 1).mul(rect.slope.x1))
        && ConstRational::new(x, 1).le(ConstRational::new(rect.depth, 1).mul(rect.slope.x2));
    let y_symmetric = ConstRational::new(y, 1)
        .ge(ConstRational::new(rect.depth, 1).mul(rect.slope.y1))
        && ConstRational::new(y, 1).le(ConstRational::new(rect.depth, 1).mul(rect.slope.y2));

    x_symmetric && y_symmetric
}

const fn transform(direction: &Direction, x: i32, y: i32, depth: i32) -> (i32, i32, i32) {
    match direction {
        Direction::Forward => (x, -depth, y),
        Direction::Back => (x, depth, y),
        Direction::Up => (x, y, depth),
        Direction::Down => (x, y, -depth),
        Direction::Right => (depth, x, y),
        Direction::Left => (-depth, x, y),
    }
}

fn cast(
    cam_pos: &Vec3<i32>,
    dir: &Direction,
    map: &WorldMap,
    sight_radius: u32,
) -> Vec<(i32, i32, i32)> {
    let mut sight_tiles = Vec::new();
    let mut rect_stack: Vec<Rect> = Vec::new();
    let init_rect = Rect::new(
        1,
        Slope {
            x1: ConstRational::new(-1, 1),
            y1: ConstRational::new(-1, 1),
            x2: ConstRational::new(1, 1),
            y2: ConstRational::new(1, 1),
        },
    );
    rect_stack.push(init_rect);

    let shift_back =
        |pos: (i32, i32, i32)| (pos.0 + cam_pos.x, pos.1 + cam_pos.y, pos.2 + cam_pos.z);
    while let Some(rect) = rect_stack.pop() {
        let (depth, (x1, y1, x2, y2)) = rect.tiles();
        let mut is_obstacle_on_prev_row: Option<bool> = None;
        let mut rect = rect.clone(); // Прямоугольник, который может быть продолжен или прерван препятствием
        for y in y1..=y2 {
            let mut is_obstacle_on_row = false;
            let mut is_prev_x_obstacle: Option<bool> = None;

            let mut str_rect = rect.next();
            for x in x1..=x2 {
                let (slope_x, slope_y) = slope(depth, x, y);
                str_rect.slope.y1 = slope_y.sub(ConstRational::new(1, 2));
                str_rect.slope.y2 = slope_y.add(ConstRational::new(1, 2));
                let radio = ((x * x + y * y + depth * depth) as f64).sqrt();
                let in_sight_radius = radio <= 1. + sight_radius as f64;

                let crds = transform(dir, x, y, depth);
                let (x_crd, y_crd, z_crd) = shift_back(crds);
                let (ch_x, ch_y, ch_z) = WorldMap::xy_chunk(x_crd, y_crd, z_crd);
                // dbg!(ch_x, ch_y, ch_z);
                let chunk_mutex = map.get_chunk(ch_x, ch_y, ch_z).unwrap();
                let chunk = chunk_mutex.lock().unwrap();
                let real_crd = WorldMap::xy_index_chunk(x_crd, y_crd, z_crd);
                let is_obstacle = chunk.obstacles[real_crd] || !in_sight_radius;
                is_obstacle_on_row = is_obstacle_on_row || is_obstacle;
                if (is_obstacle || is_symmetric(&rect, x, y)) && in_sight_radius {
                    sight_tiles.push(crds);
                }
                if let Some(is_prev_obstacle) = is_prev_x_obstacle {
                    if !is_prev_obstacle && is_obstacle && in_sight_radius {
                        // Если мы после пустого тайла встречаем стену, то режем
                        let mut new_str_rect = str_rect.next();
                        new_str_rect.slope.x2 = slope_x;
                        rect_stack.push(new_str_rect.clone());
                    }
                    if !is_obstacle && is_prev_obstacle {
                        str_rect.slope.x1 = slope_x;
                        str_rect.slope.y1 = slope_y.sub(ConstRational::new(1, 2));
                        str_rect.slope.y2 = slope_y.add(ConstRational::new(1, 2));
                    }
                }
                is_prev_x_obstacle = Some(is_obstacle);
            }

            if is_obstacle_on_row
                && is_obstacle_on_prev_row.is_some_and(|x| !x)
                && rect.depth < sight_radius as i32
            {
                // Если на этой строке нашлось препятствие, то прошлый прямоугольник обрезаем спереди и пушим
                let mut new_rect = rect.next();
                new_rect.slope.y2 = slope(depth, 1, y).1;
                rect_stack.push(new_rect);
            }
            if is_obstacle_on_prev_row.is_some_and(|x| x) && !is_obstacle_on_row {
                // Если на прошлой строке были препятствия, а на этой - нет, обрезаем сзади прямоугольник и снова тянем
                rect.slope.y1 = slope(depth, 1, y).1;
            }
            is_obstacle_on_prev_row = Some(is_obstacle_on_row);
        }
        if is_obstacle_on_prev_row.is_some_and(|x| !x) && (rect.depth as f64) < sight_radius as f64
        {
            rect_stack.push(rect.next());
        }
    }
    sight_tiles
}
