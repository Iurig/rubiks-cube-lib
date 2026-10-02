#![expect(
    clippy::panic_in_result_fn,
    reason = "`?` reports setup failures; `assert!` reports the property under test failing"
)]

mod algebra;
mod mask;
mod moves;
mod notation;
mod reconstructions;
mod solving;
mod states;

const IMPLEMENTED_MOVES: [&str; 12] = ["R", "U", "D", "L", "F", "B", "E", "S", "M", "y", "z", "x"];
