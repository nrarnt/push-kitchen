use crate::puzzle::{self, Board};

/// One kitchen of the game.
pub struct Level {
    /// Name written to the progress file. Changing it forgets that the level was solved.
    pub id: &'static str,
    /// Name shown to the player.
    pub name: &'static str,
    text: &'static str,
}

impl Level {
    pub fn board(&self) -> Board {
        puzzle::parse(self.text).expect("built-in levels are checked by the tests")
    }
}

/// Every kitchen, in playing order. `include_str!` bakes each file into the
/// program when it is compiled, so the game needs no files next to it to run.
pub const LEVELS: &[Level] = &[
    Level {
        id: "first-push",
        name: "First push",
        text: include_str!("../../assets/levels/first_push.txt"),
    },
    Level {
        id: "tomato-soup",
        name: "Tomato soup",
        text: include_str!("../../assets/levels/tomato_soup.txt"),
    },
    Level {
        id: "sandwich",
        name: "Sandwich",
        text: include_str!("../../assets/levels/sandwich.txt"),
    },
    Level {
        id: "lunch-rush",
        name: "Lunch rush",
        text: include_str!("../../assets/levels/lunch_rush.txt"),
    },
];

#[cfg(test)]
mod tests {
    use std::collections::{HashSet, VecDeque};

    use super::*;
    use crate::puzzle::{Dir, Item, Pos};

    /// Everything that can differ between two boards of the same level:
    /// where the chef is, and which item is where.
    type Key = (Pos, Vec<(Pos, Item)>);

    fn key(board: &Board) -> Key {
        let mut items: Vec<(Pos, Item)> = board.items().collect();
        // Sorted, so the same items always give the same key.
        items.sort_by_key(|(pos, _)| (pos.y, pos.x));
        (board.chef(), items)
    }

    /// Tries every sequence of moves, shortest first, until one solves the
    /// board or there is nothing new left to try.
    fn can_be_solved(start: Board) -> bool {
        let mut seen = HashSet::from([key(&start)]);
        let mut to_try = VecDeque::from([start]);

        while let Some(board) = to_try.pop_front() {
            if board.is_solved() {
                return true;
            }
            for dir in [Dir::Up, Dir::Down, Dir::Left, Dir::Right] {
                if let Some(next) = board.step(dir)
                    && seen.insert(key(&next))
                {
                    to_try.push_back(next);
                }
            }
        }
        false
    }

    #[test]
    fn the_solver_finds_a_solution() {
        assert!(can_be_solved(puzzle::parse("#@.tT#").unwrap()));
    }

    #[test]
    fn the_solver_notices_a_dead_end() {
        // The tomato is stuck against the left wall, away from its hatch.
        assert!(!can_be_solved(puzzle::parse("#t@.T#").unwrap()));
    }

    #[test]
    fn every_level_parses() {
        for level in LEVELS {
            assert!(puzzle::parse(level.text).is_ok(), "{} does not parse", level.id);
        }
    }

    #[test]
    fn level_ids_are_unique() {
        let ids: HashSet<&str> = LEVELS.iter().map(|level| level.id).collect();
        assert_eq!(ids.len(), LEVELS.len());
    }

    #[test]
    fn no_level_starts_solved() {
        for level in LEVELS {
            assert!(!level.board().is_solved(), "{} starts solved", level.id);
        }
    }

    #[test]
    #[ignore = "takes a few seconds; run with `cargo test -- --ignored` after changing a level"]
    fn every_level_can_be_solved() {
        for level in LEVELS {
            assert!(can_be_solved(level.board()), "{} cannot be solved", level.id);
        }
    }
}
