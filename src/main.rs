#![warn(
    clippy::all,
    clippy::pedantic,
    clippy::print_stdout,
    clippy::arithmetic_side_effects,
    clippy::as_conversions,
    clippy::integer_division
)]

mod editor;
mod terminal;
mod view;
use editor::Editor;

fn main() {
    let mut beppe = Editor::new().unwrap();
    beppe.run();
}
