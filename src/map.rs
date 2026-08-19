use nalgebra_glm::Vec2;

pub const CELL_SIZE: f32 = 64.0;
pub type Grid = Vec<Vec<char>>;

#[derive(Clone, Copy)]
pub struct Exit {
    pub position: Vec2,
}

#[derive(Clone, Copy)]
pub struct Collectible {
    pub position: Vec2,
}

pub struct GameMap {
    pub cells: Grid,
    pub start: Vec2,
    pub exit: Exit,
    pub collectibles: Vec<Collectible>,
    pub discovered: Vec<Vec<bool>>,
}

impl GameMap {
    pub fn load() -> Self {
        let mut cells = Vec::new();
        let mut start = Vec2::new(CELL_SIZE * 1.5, CELL_SIZE * 1.5);
        let mut exit = Exit {
            position: Vec2::new(CELL_SIZE * 22.5, CELL_SIZE * 19.5),
        };
        let mut collectibles = Vec::new();
        for (row, line) in include_str!("../map.txt").lines().enumerate() {
            let mut parsed = Vec::new();
            for (col, tile) in line.chars().enumerate() {
                let pos = Vec2::new(
                    (col as f32 + 0.5) * CELL_SIZE,
                    (row as f32 + 0.5) * CELL_SIZE,
                );
                match tile {
                    'S' => {
                        start = pos;
                        parsed.push(' ');
                    }
                    'D' => {
                        exit = Exit { position: pos };
                        parsed.push('D');
                    }
                    'B' | 'P' | 'K' => {
                        collectibles.push(Collectible { position: pos });
                        parsed.push(' ');
                    }
                    _ => parsed.push(tile),
                }
            }
            cells.push(parsed);
        }
        let discovered = vec![vec![false; cells.first().map_or(0, Vec::len)]; cells.len()];
        Self {
            cells,
            start,
            exit,
            collectibles,
            discovered,
        }
    }

    pub fn width(&self) -> usize {
        self.cells.first().map_or(0, Vec::len)
    }
    pub fn height(&self) -> usize {
        self.cells.len()
    }
    pub fn is_wall(&self, col: i32, row: i32) -> bool {
        if row < 0 || col < 0 {
            return true;
        }
        self.cells
            .get(row as usize)
            .and_then(|r| r.get(col as usize))
            .is_none_or(|c| *c != ' ')
    }
    pub fn tile_at(&self, col: i32, row: i32) -> char {
        self.cells
            .get(row as usize)
            .and_then(|r| r.get(col as usize))
            .copied()
            .unwrap_or('#')
    }
    pub fn world_to_cell(position: Vec2) -> (i32, i32) {
        (
            (position.x / CELL_SIZE) as i32,
            (position.y / CELL_SIZE) as i32,
        )
    }
    pub fn reveal_near(&mut self, position: Vec2, radius: i32) {
        let (cx, cy) = Self::world_to_cell(position);
        for row in cy - radius..=cy + radius {
            for col in cx - radius..=cx + radius {
                if row >= 0
                    && col >= 0
                    && (row as usize) < self.height()
                    && (col as usize) < self.width()
                {
                    self.discovered[row as usize][col as usize] = true;
                }
            }
        }
    }
    pub fn wall_color(tile: char) -> u32 {
        match tile {
            '#' => 0x7D2935,
            'R' => 0xA84436,
            'W' => 0xA26A3A,
            'I' => 0x2D6B7D,
            'D' => 0x4E3027,
            'B' => 0x9A4C34,
            'P' => 0x6A3D86,
            _ => 0x63303B,
        }
    }
}
