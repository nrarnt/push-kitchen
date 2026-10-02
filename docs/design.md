# Push Kitchen (working title): a calm kitchen puzzle game in Rust + Bevy

## Context

A game built from scratch in Rust and Bevy, as a way to learn Rust, for Nilo's wife to play. She likes Overcooked and old-school puzzle games (block pushing, logic/routing). Built on Rust 1.99 stable and Bevy 0.19.

Decisions:

| Topic | Decision |
|---|---|
| Players | Solo |
| Pace | Calm and thinky: no timer, no fail state |
| Puzzle style | Sokoban-style block pushing with routing through stations |
| From Overcooked | Recipe steps, combining ingredients, quirky kitchens |
| Core move | Push only (walk into an item to shove it one tile) |
| Look | 2D top-down; coloured squares first, free asset pack (e.g. Kenney) later |
| Code structure | Plain-Rust puzzle core, Bevy only draws and handles input |

Success = a small playable game she wants to pick up, and every line of it is understood.

## Game rules

- **Kitchen:** a small grid of floor, walls, stations and serving hatches. Chef moves one tile per key press (arrows / WASD).
- **Pushing:** walking into an ingredient shoves it one tile. Blocked by walls and by other items, unless a recipe combines the two.
- **Stations:** special floor tiles. An ingredient landing on one is transformed (chopping board: tomato → chopped tomato; stove: raw → cooked). Items the station has no use for pass over unchanged.
- **Combining:** pushing item A into item B merges them if a recipe exists (bread + cheese → sandwich); otherwise the push is blocked.
- **Winning:** each hatch shows the dish it wants; solved when every hatch holds its dish.
- **Safety nets:** unlimited undo (Z), restart (R).
- **Twists:** conveyor belts, ice floors, bin (see below).

### Twists

- **Ice:** an item pushed onto ice keeps sliding the same way until it leaves the ice or something stops it.
- **Conveyor:** an item on a conveyor is carried in the belt's direction, square after square, until it leaves the belt or is blocked. A blocked item rides on as soon as the way is clear. An item carried onto ice keeps sliding.
- **Bin:** an item that lands on the bin is gone.
- The chef is never affected: they walk over ice, belts and the bin like floor. Standing in a belt's way blocks it.
- A sliding or carried item combines, gets cooked or is binned by what it lands on, exactly as a pushed one.

Everything happens within the move that caused it. `Board::step` has three phases: the chef walks, the item in the way is shoved (across ice, if any), then the conveyors run until nothing moves. When several items ride at once they are handled in a fixed order (top row first, left to right), and a ring of conveyors stops after a limited number of moves.

### Recipes

| Where | From | To |
|---|---|---|
| Chopping board | tomato | chopped tomato |
| Stove | chopped tomato | tomato soup |
| Stove | sandwich | toastie |
| Pushed together | bread + cheese (either order) | sandwich |

Items pushed together end up on the square of the one that was standing still. If that square is a station, the result is cooked by it straight away (bread pushed into cheese resting on a stove gives a toastie).

### Level files

One character per square.

| Symbol | Meaning |
|---|---|
| `#` `.` `@` | wall, floor, chef |
| `/` `~` | chopping board, stove |
| `^` `v` `<` `>` | conveyor going up, down, left, right |
| `*` `x` | ice, bin |
| `t` `d` `s` | tomato, chopped (diced) tomato, tomato soup |
| `b` `c` `w` `g` | bread, cheese, sandwich, toastie (grilled) |
| capital of an item letter | hatch that wants that item (`S` wants soup) |

The chef can walk over stations and hatches. An item cannot start on a station or a hatch.

## Architecture

Two layers with a one-way dependency: Bevy code knows the puzzle core, the core knows nothing about Bevy.

```
push-kitchen/
  Cargo.toml              bevy 0.19 (2D and sound only), dev-profile speedups, web profile
  assets/levels/*.txt     ASCII kitchens (# wall, @ chef, t tomato, ...)
  src/
    main.rs               builds the Bevy App, registers plugins
    puzzle/               PURE RUST, unit tested, no Bevy imports
      mod.rs
      types.rs            Pos, Dir, Tile, StationKind, Item
      board.rs            Board + step(&self, Dir) -> Option<Board>, is_solved()
      recipes.rs          transform(station, item), combine(a, b)
      level.rs            parse(&str) -> Result<Board, LevelError>
    game/                 BEVY LAYER
      mod.rs              GamePlugin, app states (Menu / Playing), selected level
      levels.rs           the list of kitchens, baked in with include_str!
      progress.rs         which kitchens are solved, saved to a file (or the browser's storage)
      session.rs          Resource: current Board + undo history (Vec<Board>)
      input.rs            keys, swipes and button taps while playing -> move / undo / restart / next / menu
      pointer.rs          fingers and the mouse -> taps and swipes (Gesture)
      view.rs             draws the Board with sprites, slides what moved
      sound.rs            which sound a move makes, and playing it
      ui.rs               level select menu
  assets/sprites/*.png    one picture per tile, item and the chef
  assets/sounds/*.wav     one file per sound effect
  tools/make_sprites.py   draws the sprites (Python + Pillow)
  tools/make_sounds.py    synthesises the sounds (plain Python)
  tools/build_web.sh      builds the browser version into web/dist
  web/index.html          the page the browser version runs in
```

The sprites and sounds are original, made by the two scripts in `tools/`. To use other art or sounds (a Kenney pack, say), replace a file in `assets/` with one of the same name; no code changes.

Sliding: the board does not track which item is which, so `Session` keeps the board from before the latest change and the view compares the two to see what moved. Anything that moved is drawn on its old square and slides to the new one in 0.12 s.

`cargo run` finds `assets/` in the project folder. A binary started any other way looks for `assets/` next to itself.

## Web build

`tools/build_web.sh` compiles the game to WebAssembly and puts everything a web host needs into `web/dist`: `index.html`, the `.wasm` file, the JavaScript that loads it, and `assets/`. It is a folder of plain files; any static host can serve it. The script's header lists the two tools it needs.

What differs in a browser:

- The game draws into the page's `<canvas id="game">`. The camera always shows an 800 x 720 view, scaled to fit, so every kitchen and the whole menu fit any window. Tests check that each level and the menu fit that view.
- Assets are fetched over HTTP one by one, so they are preloaded by name (a browser cannot list a folder).
- Progress goes to `localStorage` instead of a file (`SaveSlot` in `progress.rs`).
- Browsers keep sound off until the first key press or click; `index.html` switches it on then.
- On a phone it is played by touch (see below). The page stops the browser from scrolling or zooming when a finger drags across the game.
- A browser stops drawing a page that is not visible, so the game stands still in a background tab and carries on when the tab is shown again.

## Touch

`pointer.rs` watches fingers (and the mouse, which counts as one more finger) and reports two kinds of `Gesture`: a tap at a point in the view, or a swipe in a direction. A swipe is reported as soon as the finger has travelled 24 screen pixels, without waiting for it to lift, and once per touch. The screens read gestures the same way they read keys.

- **In a kitchen:** a swipe anywhere moves the chef one square.
- **Touch mode:** the first time the screen is touched, the game starts showing buttons for what a keyboard has keys for. Under a kitchen: Undo, Restart (Next once it is solved) and Menu, in place of the key help. In the menu: Play.
- **In the menu:** tapping a kitchen selects it and Play starts it. A tap does not start a kitchen directly, because a phone shows the list at about half size and the lines are too close together to hit the right one every time. Swiping up or down moves the selection one line.

Buttons are 230 x 84 in the view, about 110 x 40 points on a phone. A test checks that every kitchen leaves room for them.

## Keyboard input

The game reads key presses one by one, in the order they happened (Bevy's `KeyboardInput` messages), instead of asking each frame which keys are down. That way no press is lost when several arrive in the same frame, for example after a stutter. A key that is held down counts once.

When the screen changes (menu to level, level to menu, level to next level), key presses, taps and swipes that have not been handled yet are thrown away, so one meant for the old screen is never acted on by the new one.

There is no separate "solved" state: a solved kitchen stays on screen in `Playing`, shows "Solved!", and Enter moves on to the next one.

Every kitchen can be played from the start; the menu marks the solved ones and opens on the first unsolved one. Progress is text with one level id per line. On a computer it is the file `~/Library/Application Support/Push Kitchen/progress.txt`; in a browser it is kept in the page's `localStorage`, so it belongs to that browser on that device.

Key idea: `Board::step` is a pure function (old board + direction → new board, or `None` if the move is illegal). Undo is pushing/popping boards on a `Vec`. All rules are tested with `cargo test`, no window needed.

## Build order

Each milestone ends with something runnable.

| # | Milestone | Result | Rust / Bevy ideas met |
|---|---|---|---|
| 0 | Setup | Cargo project, Bevy 0.19 window opens | cargo, crates, modules, dev-profile tuning |
| 1 | Puzzle core | Walls, chef, pushable crates, win check, level parsed from text; proven by tests | structs, enums, `match`, `Option`, borrowing, `impl`, `#[test]` |
| 2 | First playable | Coloured squares on screen, keyboard moves, undo, restart, "solved" message | ECS: components, resources, systems, queries |
| 3 | Cooking | Stations transform, recipes merge, hatches want specific dishes | enums carrying data, `HashMap`, derives, traits |
| 4 | Game shell | Several levels, level select, saved progress | `Result`, `?`, custom error type, file I/O, Bevy states |
| 5 | Look and feel | Real sprites, sliding movement, sounds | assets, `Time`, interpolation |
| 6 | Twists | Conveyors, ice, bin | iterators, refactoring `step` into phases |
| 7 | Content | 12 kitchens, playtesting, web build playable in a browser | WebAssembly, `cfg` for per-platform code, build profiles |

## Verification

- `cargo test`: every puzzle rule (move, push, blocked push, transform, combine, solved) has a unit test.
- `cargo test -- --ignored --nocapture`: searches every kitchen for a solution and prints the fewest moves each one needs. Slow, so it is left out of the plain `cargo test`; run it after adding or changing a level.
- `cargo run`: play the current level set by hand.
- `cargo clippy`: no warnings.

## Deliberately left out

Co-op, timers, 3D, a visual level editor, online features. Any of them can be revisited once milestone 7 is done.
