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

impl Dir {
    /// The direction after swapping rows and columns: what went across now
    /// goes down, and what went down now goes across.
    pub fn transposed(self) -> Dir {
        match self {
            Dir::Up => Dir::Left,
            Dir::Left => Dir::Up,
            Dir::Down => Dir::Right,
            Dir::Right => Dir::Down,
        }
    }
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
    /// Carries an item on it one square in its direction, again and again.
    Conveyor(Dir),
    /// An item pushed onto ice keeps sliding the way it was going.
    Ice,
    /// An item that lands here is gone.
    Bin,
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

impl Item {
    /// Every kind of item. A new kind has to be added here too.
    pub const ALL: [Item; 7] = [
        Item::Tomato,
        Item::ChoppedTomato,
        Item::TomatoSoup,
        Item::Bread,
        Item::Cheese,
        Item::Sandwich,
        Item::Toastie,
    ];
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn transposing_swaps_across_and_down() {
        assert_eq!(Dir::Right.transposed(), Dir::Down);
        assert_eq!(Dir::Down.transposed(), Dir::Right);
        assert_eq!(Dir::Left.transposed(), Dir::Up);
        assert_eq!(Dir::Up.transposed(), Dir::Left);
    }

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
