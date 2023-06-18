use random::Source;
use tetra::graphics::Rectangle;

pub struct Map {
    pub size: (usize, usize),
    pub tiles: Vec<Tile>,
    pub obstacles: Vec<bool>,
}

pub struct Tile {
    pub name: String,
    pub texture_name: String,
    pub texture_rect: Rectangle,
}

impl Map {
    pub fn new(width: usize, height: usize) -> Self {
        let mut tiles = Vec::new();
        let mut obstacles = Vec::new();
        let mut rnd = random::default(42);
        for i in 0..height {
            for j in 0..width {
                if i == 0
                    || j == 0
                    || i == height - 1
                    || j == width - 1
                    || rnd.read::<u32>() % 10 == 0
                {
                    let tile = Tile {
                        name: "wall".into(),
                        texture_name: "tileset".into(),
                        texture_rect: Rectangle::new(0., 0., 16., 16.),
                    };
                    tiles.push(tile);
                    obstacles.push(true);
                } else {
                    let tile = Tile {
                        name: "floor".into(),
                        texture_name: "tileset".into(),
                        texture_rect: Rectangle::new(64., 16., 16., 16.),
                    };
                    tiles.push(tile);
                    obstacles.push(false)
                }
            }
        }
        Map {
            size: (width, height),
            tiles,
            obstacles,
        }
    }
    pub fn xy_index(&self, x: i32, y: i32) -> usize {
        (x + y * self.size.0 as i32) as usize
    }
}
