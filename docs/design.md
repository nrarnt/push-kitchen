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
- **Twists (later):** conveyor belts, ice floors, bin.

## Architecture

Two layers with a one-way dependency: Bevy code knows the puzzle core, the core knows nothing about Bevy.

```
push-kitchen/
  Cargo.toml              bevy 0.19, dev-profile speedups
  assets/levels/*.txt     ASCII kitchens (# wall, @ chef, T tomato, ...)
  src/
    main.rs               builds the Bevy App, registers plugins
    puzzle/               PURE RUST, unit tested, no Bevy imports
      mod.rs
      types.rs            Pos, Dir, Tile, StationKind, Item
      board.rs            Board + step(&self, Dir) -> Option<Board>, is_solved()
      recipes.rs          transform(station, item), combine(a, b)
      level.rs            parse(&str) -> Result<Board, LevelError>
    game/                 BEVY LAYER
      mod.rs              GamePlugin, app states (Menu / Playing / Solved)
      session.rs          Resource: current Board + undo history (Vec<Board>)
      input.rs            keys -> Dir / undo / restart -> session
      view.rs             spawn + sync sprites from Board, move animation
      ui.rs               level select, "solved" screen
```

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
| 7 | Content | 10 to 15 kitchens, playtesting, macOS release build | `cargo build --release`, bundling |

## Verification

- `cargo test`: every puzzle rule (move, push, blocked push, transform, combine, solved) has a unit test.
- `cargo run`: play the current level set by hand.
- `cargo clippy`: no warnings.

## Deliberately left out

Co-op, timers, 3D, a visual level editor, online features. Any of them can be revisited once milestone 7 is done.
