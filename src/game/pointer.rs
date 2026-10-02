//! Touch and mouse: turning fingers on the screen into taps and swipes.

use std::collections::HashSet;

use bevy::prelude::*;
use bevy::window::PrimaryWindow;

use crate::puzzle::Dir;

/// How far a finger has to travel, in screen pixels, before it counts as a
/// swipe rather than a tap.
const SWIPE_DISTANCE: f32 = 24.0;

/// The mouse is treated as one more finger, with this id.
const MOUSE: u64 = u64::MAX;

/// Something the player did with a finger or the mouse.
#[derive(Message, Debug, Clone, Copy, PartialEq)]
pub enum Gesture {
    /// A touch that stayed where it was. The position is in the game's view,
    /// the same coordinates sprites and text are placed with.
    Tap(Vec2),
    Swipe(Dir),
}

/// True once the screen has been touched. From then on the game shows
/// buttons for the things a keyboard has keys for.
#[derive(Resource, Default)]
pub struct TouchMode(pub bool);

/// A rectangle in the game's view that can be tapped.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Button {
    pub label: &'static str,
    pub centre: Vec2,
    pub size: Vec2,
}

impl Button {
    pub fn contains(&self, at: Vec2) -> bool {
        let from_centre = (at - self.centre).abs();
        let half = self.size / 2.0;
        from_centre.x <= half.x && from_centre.y <= half.y
    }
}

/// The direction of a swipe from `start` to `now`, or `None` if the finger
/// has not moved far enough to count. Positions are in screen pixels, where
/// `y` grows downwards.
fn swipe(start: Vec2, now: Vec2) -> Option<Dir> {
    let moved = now - start;
    if moved.length() < SWIPE_DISTANCE {
        return None;
    }
    // Sideways or up-and-down: whichever the finger did more of.
    let dir = if moved.x.abs() > moved.y.abs() {
        if moved.x > 0.0 { Dir::Right } else { Dir::Left }
    } else if moved.y > 0.0 {
        Dir::Down
    } else {
        Dir::Up
    };
    Some(dir)
}

/// What `read_pointer` remembers from one frame to the next.
#[derive(Default)]
pub struct Tracking {
    /// Where the mouse button went down, while it is held.
    mouse_start: Option<Vec2>,
    /// Fingers that have already made their swipe and are still down.
    swiped: HashSet<u64>,
}

/// Watches fingers and the mouse, and reports each tap and swipe.
///
/// A swipe is reported as soon as the finger has travelled far enough,
/// without waiting for it to lift, and only once per touch.
pub fn read_pointer(
    touches: Res<Touches>,
    mouse: Res<ButtonInput<MouseButton>>,
    window: Single<&Window, With<PrimaryWindow>>,
    camera: Single<(&Camera, &GlobalTransform)>,
    mut touch_mode: ResMut<TouchMode>,
    mut tracking: Local<Tracking>,
    mut gestures: MessageWriter<Gesture>,
) {
    if touches.any_just_pressed() && !touch_mode.0 {
        touch_mode.0 = true;
    }

    // Every finger on the screen and every one that has just left it:
    // (id, where it went down, where it is now, whether it has lifted).
    let mut presses: Vec<(u64, Vec2, Vec2, bool)> = Vec::new();
    for touch in touches.iter() {
        presses.push((touch.id(), touch.start_position(), touch.position(), false));
    }
    for touch in touches.iter_just_released() {
        presses.push((touch.id(), touch.start_position(), touch.position(), true));
    }

    let held = mouse.pressed(MouseButton::Left);
    if mouse.just_pressed(MouseButton::Left) {
        tracking.mouse_start = window.cursor_position();
    }
    if let (Some(start), Some(cursor)) = (tracking.mouse_start, window.cursor_position()) {
        presses.push((MOUSE, start, cursor, !held));
    }
    if !held {
        tracking.mouse_start = None;
    }

    for (id, start, now, lifted) in presses {
        let direction = swipe(start, now);
        if lifted {
            // A finger that has already swiped has had its say.
            if tracking.swiped.remove(&id) {
                continue;
            }
            match direction {
                Some(dir) => {
                    gestures.write(Gesture::Swipe(dir));
                }
                None => {
                    // From a place on the screen to a place in the view.
                    let (camera, transform) = *camera;
                    if let Ok(at) = camera.viewport_to_world_2d(transform, now) {
                        gestures.write(Gesture::Tap(at));
                    }
                }
            }
        } else if let Some(dir) = direction
            && tracking.swiped.insert(id)
        {
            gestures.write(Gesture::Swipe(dir));
        }
    }

    // A touch the system took away (a phone call, say) never lifts.
    for touch in touches.iter_just_canceled() {
        tracking.swiped.remove(&touch.id());
    }
    if !held {
        tracking.swiped.remove(&MOUSE);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const START: Vec2 = Vec2::new(100.0, 100.0);

    #[test]
    fn a_finger_that_barely_moved_is_not_swiping() {
        assert_eq!(swipe(START, START + Vec2::new(5.0, -8.0)), None);
    }

    #[test]
    fn a_swipe_goes_the_way_the_finger_moved() {
        assert_eq!(swipe(START, START + Vec2::new(60.0, 0.0)), Some(Dir::Right));
        assert_eq!(swipe(START, START + Vec2::new(-60.0, 0.0)), Some(Dir::Left));
    }

    #[test]
    fn down_the_screen_is_down_in_the_kitchen() {
        // On screen, `y` grows downwards.
        assert_eq!(swipe(START, START + Vec2::new(0.0, 60.0)), Some(Dir::Down));
        assert_eq!(swipe(START, START + Vec2::new(0.0, -60.0)), Some(Dir::Up));
    }

    #[test]
    fn a_slanted_swipe_takes_its_main_direction() {
        assert_eq!(swipe(START, START + Vec2::new(50.0, 20.0)), Some(Dir::Right));
        assert_eq!(swipe(START, START + Vec2::new(-15.0, -45.0)), Some(Dir::Up));
    }

    const BUTTON: Button = Button {
        label: "Go",
        centre: Vec2::new(10.0, -20.0),
        size: Vec2::new(120.0, 50.0),
    };

    #[test]
    fn a_button_contains_its_centre_and_edges() {
        let button = BUTTON;
        assert!(button.contains(button.centre));
        assert!(button.contains(button.centre + button.size / 2.0));
        assert!(button.contains(button.centre - button.size / 2.0));
    }

    #[test]
    fn a_button_does_not_contain_points_beside_it() {
        let button = BUTTON;
        let half = button.size / 2.0;
        assert!(!button.contains(button.centre + Vec2::new(half.x + 1.0, 0.0)));
        assert!(!button.contains(button.centre - Vec2::new(0.0, half.y + 1.0)));
    }
}
