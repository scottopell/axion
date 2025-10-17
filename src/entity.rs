#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Position {
    pub x: i32,
    pub y: i32,
}

impl Position {
    pub fn new(x: i32, y: i32) -> Self {
        Self { x, y }
    }

    pub fn moved(&self, direction: Direction) -> Self {
        match direction {
            Direction::Up => Position::new(self.x, self.y - 1),
            Direction::Down => Position::new(self.x, self.y + 1),
            Direction::Left => Position::new(self.x - 1, self.y),
            Direction::Right => Position::new(self.x + 1, self.y),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Direction {
    Up,
    Down,
    Left,
    Right,
}

impl Direction {
    pub fn opposite(&self) -> Direction {
        match self {
            Direction::Up => Direction::Down,
            Direction::Down => Direction::Up,
            Direction::Left => Direction::Right,
            Direction::Right => Direction::Left,
        }
    }
}

#[derive(Debug, Clone)]
pub struct Player {
    pub position: Position,
    pub direction: Direction,
    pub trail: Vec<Position>,
    pub is_drawing: bool,
}

impl Player {
    pub fn new(x: i32, y: i32) -> Self {
        Self {
            position: Position::new(x, y),
            direction: Direction::Right,
            trail: Vec::new(),
            is_drawing: false,
        }
    }

    pub fn start_trail(&mut self) {
        self.is_drawing = true;
        self.trail.clear();
        self.trail.push(self.position);
    }

    pub fn add_to_trail(&mut self) {
        if self.is_drawing {
            self.trail.push(self.position);
        }
    }

    pub fn clear_trail(&mut self) {
        self.trail.clear();
        self.is_drawing = false;
    }
}

pub trait Enemy {
    fn position(&self) -> Position;
    fn update(&mut self, width: i32, height: i32, is_filled: &dyn Fn(i32, i32) -> bool);
}

#[derive(Debug, Clone)]
pub struct Ball {
    pub position: Position,
    pub velocity: (i32, i32),
}

impl Ball {
    pub fn new(x: i32, y: i32, vx: i32, vy: i32) -> Self {
        Self {
            position: Position::new(x, y),
            velocity: (vx, vy),
        }
    }
}

impl Enemy for Ball {
    fn position(&self) -> Position {
        self.position
    }

    fn update(&mut self, width: i32, height: i32, is_filled: &dyn Fn(i32, i32) -> bool) {
        let mut next_x = self.position.x + self.velocity.0;
        let mut next_y = self.position.y + self.velocity.1;

        // Bounce off walls or filled cells
        if next_x <= 0 || next_x >= width - 1 || is_filled(next_x, self.position.y) {
            self.velocity.0 = -self.velocity.0;
            next_x = self.position.x + self.velocity.0;
        }

        if next_y <= 0 || next_y >= height - 1 || is_filled(self.position.x, next_y) {
            self.velocity.1 = -self.velocity.1;
            next_y = self.position.y + self.velocity.1;
        }

        self.position.x = next_x;
        self.position.y = next_y;
    }
}
