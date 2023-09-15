use hecs::World;

use crate::{
    entities::{Player, Position, Sight},
    map::Map,
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

pub fn run_fov_compute_system(world: &World) {
    if let Some((_, (map,))) = world.query::<(&mut Map,)>().iter().next() {
        if let Some((_, (_, Position(cam_pos), Sight(sight_tiles)))) = world
            .query::<(&Player, &Position, &mut Sight)>()
            .iter()
            .next()
        {
            // относительные координаты в реальные
            let shift_back = |pos: (i32, i32)| (pos.0 + cam_pos.x, pos.1 + cam_pos.y);

            sight_tiles.clear();
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
                    let sight_radius = 30.0;
                    let mut row = row;
                    let mut is_prev_obstacle: Option<bool> = None;
                    for (depth, col) in row.clone().tiles() {
                        let hypotenuse = ((col * col + depth * depth) as f64).sqrt();
                        let in_sight_radius = hypotenuse < sight_radius;

                        let crds = transform(&dir, col, depth);
                        let (x, y) = shift_back(crds);
                        let (ch_x, ch_y) = Map::xy_chunk(x, y);
                        let chunk = map.get_chunk_or_create(ch_x, ch_y);
                        let real_crd = Map::xy_index_chunk(x, y);
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
                        //&& !row.tiles().collect::<Vec<(i32, i32)>>().is_empty()
                        && (row.depth as f64) < sight_radius
                    {
                        row_stack.push(row.next());
                    }
                }
            }
        }
    }
}
