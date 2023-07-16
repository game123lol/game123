use random::Source;
use tetra::graphics::Rectangle;

pub struct Tile {
    pub name: String,
    pub texture_name: String,
    pub texture_rect: Rectangle,
}

impl Tile {
    pub fn new(name: &str, texture_name: &str, x: u8, y: u8) -> Self {
        Tile {
            name: name.into(),
            texture_name: texture_name.into(),
            texture_rect: Rectangle::new(x as f32 * 16., y as f32 * 20., 16., 20.),
        }
    }
}

pub struct Map {
    pub size: (usize, usize),
    pub tiles: Vec<Tile>,
    pub obstacles: Vec<bool>,
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
                    let tile = Tile::new("wall", "tileset_iso", 0, 0);
                    tiles.push(tile);
                    obstacles.push(true);
                } else {
                    let tile = Tile::new("wall", "tileset_iso", 0, 1);
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
