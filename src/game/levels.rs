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
    Level {
        id: "conveyor",
        name: "Conveyor",
        text: include_str!("../../assets/levels/conveyor.txt"),
    },
    Level {
        id: "slippery-floor",
        name: "Slippery floor",
        text: include_str!("../../assets/levels/slippery_floor.txt"),
    },
    Level {
        id: "leftovers",
        name: "Leftovers",
        text: include_str!("../../assets/levels/leftovers.txt"),
    },
    Level {
        id: "mind-the-gap",
        name: "Mind the gap",
        text: include_str!("../../assets/levels/mind_the_gap.txt"),
    },
    Level {
        id: "cheese-on-ice",
        name: "Cheese on ice",
        text: include_str!("../../assets/levels/cheese_on_ice.txt"),
    },
    Level {
        id: "ice-rink",
        name: "Ice rink",
        text: include_str!("../../assets/levels/ice_rink.txt"),
    },
    Level {
        id: "soup-line",
        name: "Soup line",
        text: include_str!("../../assets/levels/soup_line.txt"),
    },
    Level {
        id: "dinner-service",
        name: "Dinner service",
        text: include_str!("../../assets/levels/dinner_service.txt"),
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
    /// board or there is nothing new left to try. Returns how many moves the
    /// shortest solution takes, or `None` if there is no solution.
    fn fewest_moves(start: Board) -> Option<usize> {
        let mut seen = HashSet::from([key(&start)]);
        let mut to_try = VecDeque::from([(start, 0)]);

        while let Some((board, moves)) = to_try.pop_front() {
            if board.is_solved() {
                return Some(moves);
            }
            for dir in [Dir::Up, Dir::Down, Dir::Left, Dir::Right] {
                if let Some(next) = board.step(dir)
                    && seen.insert(key(&next))
                {
                    to_try.push_back((next, moves + 1));
                }
            }
        }
        None
    }

    #[test]
    fn the_solver_finds_a_solution() {
        assert_eq!(fewest_moves(puzzle::parse("#@.tT#").unwrap()), Some(2));
    }

    #[test]
    fn the_solver_notices_a_dead_end() {
        // The tomato is stuck against the left wall, away from its hatch.
        assert_eq!(fewest_moves(puzzle::parse("#t@.T#").unwrap()), None);
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
    #[ignore = "slow; run with `cargo test -- --ignored --nocapture` after changing a level"]
    fn every_level_can_be_solved() {
        for level in LEVELS {
            let moves = fewest_moves(level.board());
            assert!(moves.is_some(), "{} cannot be solved", level.id);
            // Shown with --nocapture: a rough measure of how hard each level is.
            println!("{:16} {:3} moves", level.id, moves.unwrap());
        }
    }
}
