#![warn(
    clippy::all,
    clippy::pedantic,
    clippy::print_stdout,
    clippy::arithmetic_side_effects,
    clippy::as_conversions,
    clippy::integer_division
)]

mod editor;
use editor::Editor;

fn main() {
    let mut beppe = Editor::new().unwrap();
    beppe.run();
}

// TODO: Make this an effective text editor
