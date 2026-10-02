use bevy::prelude::*;

use crate::puzzle::{Board, Dir};

/// The kitchen being played, plus every earlier board so changes can be undone.
#[derive(Resource)]
pub struct Session {
    board: Board,
    /// The board before each change, oldest first.
    history: Vec<Board>,
    /// The board the latest step, undo or restart changed, or `None` if it
    /// changed nothing. Lets the view show what moved.
    previous: Option<Board>,
}

impl Session {
    pub fn new(board: Board) -> Self {
        Session {
            board,
            history: Vec::new(),
            previous: None,
        }
    }

    pub fn board(&self) -> &Board {
        &self.board
    }

    pub fn previous(&self) -> Option<&Board> {
        self.previous.as_ref()
    }

    /// Moves the chef one square in `dir`, if the puzzle rules allow it.
    pub fn step(&mut self, dir: Dir) {
        self.previous = None;
        if let Some(next) = self.board.step(dir) {
            self.change_to(next);
        }
    }

    /// Takes back the latest change. A burnt kitchen cannot be taken back.
    pub fn undo(&mut self) {
        self.previous = None;
        if self.board.is_burnt() {
            return;
        }
        if let Some(earlier) = self.history.pop() {
            let undone = std::mem::replace(&mut self.board, earlier);
            self.previous = Some(undone);
        }
    }

    /// Goes back to the starting board. Counts as a change, so it can be
    /// undone too, unless the kitchen was burnt: then it is a fresh start.
    pub fn restart(&mut self) {
        self.previous = None;
        // The oldest board in the history is the one the level started with.
        // No history means we are still on it.
        if let Some(start) = self.history.first().cloned() {
            let burnt = self.board.is_burnt();
            self.change_to(start);
            if burnt {
                self.history.clear();
            }
        }
    }

    /// Makes `next` the current board and remembers the one it replaces.
    fn change_to(&mut self, next: Board) {
        let replaced = std::mem::replace(&mut self.board, next);
        self.previous = Some(replaced.clone());
        self.history.push(replaced);
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
    fn a_new_session_has_no_previous_board() {
        assert_eq!(session("#@..#").previous(), None);
    }

    #[test]
    fn a_move_remembers_the_board_before_it() {
        let mut session = session("#@..#");
        session.step(Dir::Right);
        assert_eq!(session.previous(), Some(&board("#@..#")));
    }

    #[test]
    fn an_illegal_move_has_no_previous_board() {
        let mut session = session("#@..#");
        session.step(Dir::Right);
        session.step(Dir::Up);
        assert_eq!(session.previous(), None);
    }

    #[test]
    fn undo_remembers_the_board_it_took_back() {
        let mut session = session("#@..#");
        session.step(Dir::Right);
        session.undo();
        assert_eq!(session.previous(), Some(&board("#.@.#")));
    }

    #[test]
    fn undo_with_nothing_to_undo_has_no_previous_board() {
        let mut session = session("#@..#");
        session.step(Dir::Right);
        session.undo();
        session.undo();
        assert_eq!(session.previous(), None);
    }

    #[test]
    fn restart_remembers_the_board_it_left() {
        let mut session = session("#@..#");
        session.step(Dir::Right);
        session.step(Dir::Right);
        session.restart();
        assert_eq!(session.previous(), Some(&board("#..@#")));
    }

    /// A session whose chef has just walked onto the stove.
    fn burnt_session() -> Session {
        let mut session = session("#.@~#");
        session.step(Dir::Left);
        session.step(Dir::Right);
        session.step(Dir::Right);
        assert!(session.board().is_burnt());
        session
    }

    #[test]
    fn a_burnt_kitchen_cannot_be_undone() {
        let mut session = burnt_session();
        session.undo();
        assert!(session.board().is_burnt());
    }

    #[test]
    fn restart_brings_a_burnt_kitchen_back() {
        let mut session = burnt_session();
        session.restart();
        assert_eq!(session.board(), &board("#.@~#"));
    }

    #[test]
    fn a_restart_after_burning_cannot_be_undone() {
        let mut session = burnt_session();
        session.restart();
        session.undo();
        assert_eq!(session.board(), &board("#.@~#"));
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
