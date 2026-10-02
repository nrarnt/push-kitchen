use bevy::prelude::*;

use crate::puzzle::{Board, Item};

/// A sound effect.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Sfx {
    Step,
    Push,
    Chop,
    Sizzle,
    Combine,
    /// An item dropping into the bin.
    Bin,
    /// Walking into something that will not move.
    Bump,
    Solved,
}

/// The sound's file, inside the `assets` folder.
fn file(sfx: Sfx) -> &'static str {
    match sfx {
        Sfx::Step => "sounds/step.wav",
        Sfx::Push => "sounds/push.wav",
        Sfx::Chop => "sounds/chop.wav",
        Sfx::Sizzle => "sounds/sizzle.wav",
        Sfx::Combine => "sounds/combine.wav",
        Sfx::Bin => "sounds/bin.wav",
        Sfx::Bump => "sounds/bump.wav",
        Sfx::Solved => "sounds/solved.wav",
    }
}

/// The sound of the chef's move that turned `before` into `after`.
pub fn sound_of_move(before: &Board, after: &Board) -> Sfx {
    if after.is_solved() && !before.is_solved() {
        return Sfx::Solved;
    }
    // Was there an item on the square the chef walked onto?
    let Some(pushed) = before.item(after.chef()) else {
        return Sfx::Step;
    };
    // Where did it end up? Look for an item that was not there before.
    let landed = after
        .items()
        .find(|&(pos, item)| before.item(pos) != Some(item));

    match landed {
        None => Sfx::Bin,
        Some(_) if after.items().count() < before.items().count() => Sfx::Combine,
        Some((_, item)) if item == pushed => Sfx::Push,
        // It changed on the way, so a station cooked it.
        Some((_, Item::ChoppedTomato)) => Sfx::Chop,
        Some(_) => Sfx::Sizzle,
    }
}

/// Plays a sound once. The entity that plays it removes itself when done.
pub fn play(commands: &mut Commands, assets: &AssetServer, sfx: Sfx) {
    commands.spawn((AudioPlayer::new(assets.load(file(sfx))), PlaybackSettings::DESPAWN));
}

#[cfg(test)]
mod tests {
    use std::path::Path;

    use super::*;
    use crate::puzzle::{Dir, parse};

    /// The sound of pushing right once from `text`.
    fn sound_of_right(text: &str) -> Sfx {
        let before = parse(text).expect("test level should parse");
        let after = before.step(Dir::Right).expect("move should be allowed");
        sound_of_move(&before, &after)
    }

    #[test]
    fn walking_makes_a_step() {
        assert_eq!(sound_of_right("#@.t#"), Sfx::Step);
    }

    #[test]
    fn pushing_an_item_makes_a_push() {
        assert_eq!(sound_of_right("#@t.#"), Sfx::Push);
    }

    #[test]
    fn an_item_resting_on_a_station_it_has_no_use_for_is_just_a_push() {
        assert_eq!(sound_of_right("#@b/#"), Sfx::Push);
    }

    #[test]
    fn the_chopping_board_chops() {
        assert_eq!(sound_of_right("#@t/#"), Sfx::Chop);
    }

    #[test]
    fn the_stove_sizzles() {
        assert_eq!(sound_of_right("#@d~#"), Sfx::Sizzle);
    }

    #[test]
    fn two_items_combine() {
        assert_eq!(sound_of_right("#@bc#"), Sfx::Combine);
    }

    #[test]
    fn an_item_sliding_over_ice_is_just_a_push() {
        assert_eq!(sound_of_right("#@t**..#"), Sfx::Push);
    }

    #[test]
    fn a_conveyor_carrying_a_tomato_to_the_chopping_board_chops() {
        assert_eq!(sound_of_right("#@t>/#"), Sfx::Chop);
    }

    #[test]
    fn an_item_dropping_into_the_bin_sounds_binned() {
        assert_eq!(sound_of_right("#@tx#"), Sfx::Bin);
    }

    #[test]
    fn the_move_that_solves_the_level_sounds_solved() {
        assert_eq!(sound_of_right("#@tT#"), Sfx::Solved);
    }

    #[test]
    fn walking_about_a_solved_level_is_just_steps() {
        let solved = parse("#@tT.#").unwrap().step(Dir::Right).unwrap();
        let after = solved.step(Dir::Left).unwrap();
        assert_eq!(sound_of_move(&solved, &after), Sfx::Step);
    }

    #[test]
    fn every_sound_has_its_file() {
        let all = [
            Sfx::Step,
            Sfx::Push,
            Sfx::Chop,
            Sfx::Sizzle,
            Sfx::Combine,
            Sfx::Bin,
            Sfx::Bump,
            Sfx::Solved,
        ];
        for sfx in all {
            let path = Path::new("assets").join(file(sfx));
            assert!(path.is_file(), "{} is missing", path.display());
        }
    }
}
