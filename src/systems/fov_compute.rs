use std::collections::BTreeSet;

use hecs::World;
use tetra::math::Vec2;

use crate::{
    components::{Player, Position, Sight},
    map::{Map, WorldMap},
    need_components,
};

use super::WorldSystem;

#[derive(Clone, Debug)]
struct Row {
    depth: i32,
    slope: (f64, f64),
}

enum Direction {
    Up,
    Left,
    Down,
    Right,
}

pub struct FovComputeSystem;

impl WorldSystem for FovComputeSystem {
    fn run(&self, world: &World, _ctx: &tetra::Context) -> super::Result {
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

        for dir in vec![
            Direction::Up,
            Direction::Left,
            Direction::Down,
            Direction::Right,
        ] {
            cast(cam_pos, &dir, map, sight_tiles, *sight_radius);
        }
        Ok(())
    }
}

impl Row {
    fn new(depth: i32, slope: (f64, f64)) -> Self {
        Row { depth, slope }
    }

    fn tiles(&self) -> Box<dyn Iterator<Item = (i32, i32)> + '_> {
        let min_col = (self.slope.0 * self.depth as f64 + 0.5).floor() as i32;
        let max_col = (self.slope.1 * self.depth as f64 + 0.5).floor() as i32;
        Box::new((min_col..=max_col).map(|col| (self.depth, col)))
    }
    fn next(&self) -> Self {
        Row::new(self.depth + 1, self.slope)
    }
}

fn slope(depth: i32, col: i32) -> f64 {
    (2.0 * col as f64 - 1.0) / (2.0 * depth as f64)
}

fn is_symmetric(row: &Row, col: i32) -> bool {
    col as f64 >= row.depth as f64 * row.slope.0 && col as f64 <= row.depth as f64 * row.slope.1
}

fn transform(direction: &Direction, col: i32, row: i32) -> (i32, i32) {
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
    map: &mut WorldMap,
    sight_tiles: &mut BTreeSet<(i32, i32)>,
    sight_radius: u32,
) {
    let mut row_stack: Vec<Row> = Vec::new();
    row_stack.push(Row::new(1, (-1., 1.)));
    while let Some(row) = row_stack.pop() {
        let mut row = row;
        let shift_back = |pos: (i32, i32)| (pos.0 + cam_pos.x, pos.1 + cam_pos.y);
        let mut is_prev_obstacle: Option<bool> = None;
        for (depth, col) in row.clone().tiles() {
            let hypotenuse = ((col * col + depth * depth) as f64).sqrt();
            let in_sight_radius = hypotenuse < sight_radius as f64;

            let crds = transform(dir, col, depth);
            let (x, y) = shift_back(crds);
            let (ch_x, ch_y) = WorldMap::xy_chunk(x, y);
            let chunk = map.get_chunk_or_create(ch_x, ch_y);
            let real_crd = WorldMap::xy_index_chunk(x, y);
            let is_obstacle = chunk.obstacles[real_crd];
            if (is_obstacle || is_symmetric(&row, col)) && in_sight_radius {
                sight_tiles.insert(crds);
            }
            if let Some(is_prev_obstacle) = is_prev_obstacle {
                if !is_prev_obstacle && is_obstacle && in_sight_radius {
                    let mut next_row = row.next();
                    next_row.slope.1 = slope(depth, col);
                    row_stack.push(next_row);
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
}
