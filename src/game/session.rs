use bevy::prelude::*;

use crate::puzzle::{Board, Dir};

/// The kitchen being played, plus every earlier board so changes can be undone.
#[derive(Resource)]
pub struct Session {
    board: Board,
    /// The board before each change, oldest first.
    history: Vec<Board>,
}

impl Session {
    pub fn new(board: Board) -> Self {
        Session {
            board,
            history: Vec::new(),
        }
    }

    pub fn board(&self) -> &Board {
        &self.board
    }

    /// Moves the chef one square in `dir`, if the puzzle rules allow it.
    pub fn step(&mut self, dir: Dir) {
        if let Some(next) = self.board.step(dir) {
            self.change_to(next);
        }
    }

    /// Takes back the latest change.
    pub fn undo(&mut self) {
        if let Some(previous) = self.history.pop() {
            self.board = previous;
        }
    }

    /// Goes back to the starting board. Counts as a change, so it can be undone too.
    pub fn restart(&mut self) {
        // The oldest board in the history is the one the level started with.
        // No history means we are still on it.
        if let Some(start) = self.history.first().cloned() {
            self.change_to(start);
        }
    }

    /// Makes `next` the current board and remembers the one it replaces.
    fn change_to(&mut self, next: Board) {
        let previous = std::mem::replace(&mut self.board, next);
        self.history.push(previous);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::puzzle::parse;

    fn board(text: &str) -> Board {
        parse(text).expect("test level should parse")
    }

    fn session(text: &str) -> Session {
        Session::new(board(text))
    }

    #[test]
    fn a_legal_move_changes_the_board() {
        let mut session = session("#@..#");
        session.step(Dir::Right);
        assert_eq!(session.board(), &board("#.@.#"));
    }

    #[test]
    fn an_illegal_move_leaves_the_board_alone() {
        let mut session = session("#@..#");
        session.step(Dir::Left);
        assert_eq!(session.board(), &board("#@..#"));
    }

    #[test]
    fn undo_takes_back_one_move() {
        let mut session = session("#@..#");
        session.step(Dir::Right);
        session.step(Dir::Right);
        session.undo();
        assert_eq!(session.board(), &board("#.@.#"));
    }

    #[test]
    fn undo_skips_moves_that_were_not_allowed() {
        let mut session = session("#@..#");
        session.step(Dir::Right);
        session.step(Dir::Up);
        session.undo();
        assert_eq!(session.board(), &board("#@..#"));
    }

    #[test]
    fn undo_with_nothing_to_undo_does_nothing() {
        let mut session = session("#@..#");
        session.undo();
        assert_eq!(session.board(), &board("#@..#"));
    }

    #[test]
    fn restart_goes_back_to_the_starting_board() {
        let mut session = session("#@..#");
        session.step(Dir::Right);
        session.step(Dir::Right);
        session.restart();
        assert_eq!(session.board(), &board("#@..#"));
    }

    #[test]
    fn restart_can_be_undone() {
        let mut session = session("#@..#");
        session.step(Dir::Right);
        session.step(Dir::Right);
        session.restart();
        session.undo();
        assert_eq!(session.board(), &board("#..@#"));
    }
}
