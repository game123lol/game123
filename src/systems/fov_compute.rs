use hecs::World;
use tetra::Context;

use crate::{
    entities::{Player, Position, Sight},
    map::{Map, Tile},
};

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

impl<'a> Row {
    fn new(depth: i32, slope: (f64, f64)) -> Self {
        Row { depth, slope }
    }

    fn tiles(&self) -> Box<dyn Iterator<Item = (i32, i32)> + '_> {
        let min_col = (self.slope.0 * self.depth as f64 + 0.5).floor() as i32;
        let max_col = (self.slope.1 * self.depth as f64).floor() as i32;
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

pub fn run_fov_compute_system(world: &World, ctx: &mut Context) {
    if let Some((_, (map,))) = world.query::<(&Map,)>().iter().next() {
        if let Some((_, (_, Position(cam_pos), Sight(sight_tiles)))) = world
            .query::<(&Player, &Position, &mut Sight)>()
            .iter()
            .next()
        {
            let (map_h, map_w) = map.size;

            // реальные координаты в относительные
            let shift = |pos: (i32, i32)| (pos.0 - cam_pos.x, pos.1 - cam_pos.y);
            // обратно
            let shift_back = |pos: (i32, i32)| (pos.0 + cam_pos.x, pos.1 + cam_pos.y);

            sight_tiles.insert((0, 0));

            let mut row_stack: Vec<Row> = Vec::new();

            for dir in vec![
                Direction::Up,
                Direction::Left,
                Direction::Down,
                Direction::Right,
            ] {
                row_stack.push(Row::new(1, (-1., 1.)));
                while let Some(row) = row_stack.pop() {
                    let mut is_prev_obstacle = false;
                    for (depth, col) in row.tiles() {
                        let (x, y) = shift_back(transform(&dir, col, depth));
                        let real_crd = map.xy_index_safe(x, y);
                        let is_obstacle = !(real_crd.is_some() && map.obstacles[real_crd.unwrap()]);
                        if is_obstacle || is_symmetric(&row, col) {
                            sight_tiles.insert((x, y));
                        }
                        if !is_prev_obstacle && is_obstacle {
                            let mut next_row = row.next();
                            next_row.slope.1 = slope(depth, col);
                            row_stack.push(next_row);
                            println!("left");
                        }
                        if !is_obstacle && is_prev_obstacle {
                            let mut next_row = row.next();
                            next_row.slope.0 = slope(depth, col);
                            row_stack.push(next_row);
                            println!("right");
                        }
                        is_prev_obstacle = is_obstacle;
                    }
                    if !is_prev_obstacle {
                        row_stack.push(row.next());
                    }
                }
            }
        }
    }
}
