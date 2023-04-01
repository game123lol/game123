use tetra::graphics::Rectangle;

pub struct Map {
    pub size: (usize, usize),
    pub tiles: Vec<Tile>,
}

pub struct Tile {
    pub name: String,
    pub texture_name: String,
    pub texture_rect: Rectangle,
}

impl Map {
    pub fn new(width: usize, height: usize) -> Self {
        let mut tiles = Vec::new();
        for i in 0..height {
            for j in 0..height {
                if i == 0 || j == 0 || i == width - 1 || j == height - 1 {
                    let tile = Tile {
                        name: "wall".into(),
                        texture_name: "tileset".into(),
                        texture_rect: Rectangle::new(0., 0., 16., 16.),
                    };
                    tiles.push(tile);
                } else {
                    let tile = Tile {
                        name: "floor".into(),
                        texture_name: "tileset".into(),
                        texture_rect: Rectangle::new(64., 16., 16., 16.),
                    };
                    tiles.push(tile);
                }
            }
        }
        Map {
            size: (width, height),
            tiles: vec![],
        }
    }
}
