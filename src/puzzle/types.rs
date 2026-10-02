/// A square on the kitchen grid. `y` grows downwards, like rows in a level file.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Pos {
    pub x: i32,
    pub y: i32,
}

impl Pos {
    pub fn new(x: i32, y: i32) -> Self {
        Pos { x, y }
    }

    /// The neighbouring square one step in `dir`.
    pub fn step(self, dir: Dir) -> Pos {
        match dir {
            Dir::Up => Pos::new(self.x, self.y - 1),
            Dir::Down => Pos::new(self.x, self.y + 1),
            Dir::Left => Pos::new(self.x - 1, self.y),
            Dir::Right => Pos::new(self.x + 1, self.y),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Dir {
    Up,
    Down,
    Left,
    Right,
}

/// What the floor is made of at one square.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Tile {
    Floor,
    Wall,
    /// Floor that cooks: an item landing here may be turned into another.
    Station(StationKind),
    /// Serving hatch, carrying the dish it wants. The level is solved when
    /// every hatch holds its dish.
    Hatch(Item),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StationKind {
    ChoppingBoard,
    Stove,
}

/// Something the chef can push.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Item {
    Tomato,
    ChoppedTomato,
    TomatoSoup,
    Bread,
    Cheese,
    Sandwich,
    Toastie,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn step_up_decreases_y() {
        assert_eq!(Pos::new(2, 2).step(Dir::Up), Pos::new(2, 1));
    }

    #[test]
    fn step_down_increases_y() {
        assert_eq!(Pos::new(2, 2).step(Dir::Down), Pos::new(2, 3));
    }

    #[test]
    fn step_left_decreases_x() {
        assert_eq!(Pos::new(2, 2).step(Dir::Left), Pos::new(1, 2));
    }

    #[test]
    fn step_right_increases_x() {
        assert_eq!(Pos::new(2, 2).step(Dir::Right), Pos::new(3, 2));
    }
}
