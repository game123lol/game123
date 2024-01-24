use std::sync::Mutex;

use hecs::World;
use tetra::math::Vec2;

use crate::{
    components::{Player, Position, Sight},
    map::{Chunk, Map, WorldMap},
    need_components,
};

use rationals::ConstRational;

#[derive(Clone, Debug)]
struct Row {
    depth: i32,
    slope: (ConstRational, ConstRational),
}

enum Direction {
    Up,
    Left,
    Down,
    Right,
}

pub fn run_fov_compute_system(world: &World, _ctx: &tetra::Context) -> super::Result {
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
    sight_tiles.insert((0, 0));

    let dirs = [
        Direction::Up,
        Direction::Left,
        Direction::Down,
        Direction::Right,
    ];
    let chunks_depth = (*sight_radius / 15 + 3) as i32;
    let current_chunk = WorldMap::xy_chunk(cam_pos.x, cam_pos.y);
    for i in -chunks_depth..chunks_depth {
        for j in -chunks_depth..chunks_depth {
            map.get_chunk_or_create(current_chunk.0 + i, current_chunk.1 + j);
        }
    }
    sight_tiles.extend(
        dirs.iter()
            .flat_map(|dir| cast(cam_pos, dir, map, *sight_radius))
            .collect::<Vec<(i32, i32)>>(),
    );
    Ok(())
}

impl Row {
    const fn new(depth: i32, slope: (ConstRational, ConstRational)) -> Self {
        Row { depth, slope }
    }

    fn tiles(&self) -> Box<dyn Iterator<Item = (i32, i32)> + '_> {
        let min_col = (self.slope.0.mul(ConstRational::new(self.depth, 1))).floor();
        let max_col = (self
            .slope
            .1
            .mul(ConstRational::new(self.depth, 1))
            .add(ConstRational::new(1, 2)))
        .floor();
        Box::new((min_col..=max_col).map(|col| (self.depth, col)))
    }
    const fn next(&self) -> Self {
        Row::new(self.depth + 1, self.slope)
    }
}

pub const fn slope(depth: i32, col: i32) -> ConstRational {
    ConstRational::new(2 * col - 1, 2 * depth)
}

const fn is_symmetric(row: &Row, col: i32) -> bool {
    ConstRational::new(col, 1).ge(ConstRational::new(row.depth, 1).mul(row.slope.0))
        && ConstRational::new(col, 1).le(ConstRational::new(row.depth, 1).mul(row.slope.1))
}

const fn transform(direction: &Direction, col: i32, row: i32) -> (i32, i32) {
    match direction {
        Direction::Up => (col, -row),
        Direction::Down => (col, row),
        Direction::Left => (row, col),
        Direction::Right => (-row, col),
    }
}

fn cast(
    cam_pos: &Vec2<i32>,
    dir: &Direction,
    map: &WorldMap,
    sight_radius: u32,
) -> Vec<(i32, i32)> {
    let mut sight_tiles = Vec::new();
    let mut row_stack: Vec<Row> = Vec::new();
    let mut chunk_cache: Vec<((i32, i32), *const Mutex<Chunk>)> = Vec::new();
    row_stack.push(Row::new(
        1,
        (ConstRational::new(-1, 1), ConstRational::new(1, 1)),
    ));
    while let Some(row) = row_stack.pop() {
        let mut row = row;
        let shift_back = |pos: (i32, i32)| (pos.0 + cam_pos.x, pos.1 + cam_pos.y);
        let mut is_prev_obstacle: Option<bool> = None;
        for (depth, col) in row.clone().tiles() {
            let hypotenuse = ((col * col + depth * depth) as f64).sqrt();
            let in_sight_radius = hypotenuse <= sight_radius as f64;

            let crds = transform(dir, col, depth);
            let (x, y) = shift_back(crds);
            let (ch_x, ch_y) = WorldMap::xy_chunk(x, y);
            let chunk_mutex =
                if let Some((_, chunk)) = chunk_cache.iter().rev().find(|a| a.0 == (ch_x, ch_y)) {
                    unsafe { chunk.as_ref().unwrap() }
                } else {
                    let link = map.get_chunk(ch_x, ch_y).unwrap();
                    chunk_cache.push(((ch_x, ch_y), link));
                    link
                };
            let chunk = chunk_mutex.lock().unwrap();
            let real_crd = WorldMap::xy_index_chunk(x, y);
            let is_obstacle = chunk.obstacles[real_crd];
            if (is_obstacle || is_symmetric(&row, col)) && in_sight_radius {
                sight_tiles.push(crds);
            }
            if let Some(is_prev_obstacle) = is_prev_obstacle {
                if !is_prev_obstacle && is_obstacle {
                    if row.depth < sight_radius as i32 {
                        let mut next_row = row.next();
                        next_row.slope.1 = slope(depth, col);
                        row_stack.push(next_row);
                    }
                }
                if !is_obstacle && is_prev_obstacle {
                    row.slope.0 = slope(depth, col);
                }
            }
            is_prev_obstacle = Some(is_obstacle);
        }
        if is_prev_obstacle.is_some()
            && !is_prev_obstacle.unwrap()
            && (row.depth as f64) < sight_radius as f64
        {
            row_stack.push(row.next());
        }
    }
    sight_tiles
}
