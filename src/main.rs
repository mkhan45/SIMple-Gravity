// bevy
#![allow(clippy::too_many_arguments)]
#![allow(clippy::type_complexity)]

use egui_macroquad::macroquad;
use macroquad::prelude::*;

pub mod error;
use error::SimError;

pub mod camera;
pub mod draw;
pub mod force_lines;
pub mod main_state;
pub mod physics;
pub mod preview;
pub mod scripting;
pub mod trails;
pub mod ui;

#[macroquad::main("Gravity")]
async fn main() -> Result<(), SimError> {
    next_frame().await;

    let mut main_state = main_state::MainState::default();

    loop {
        main_state.update()?;
        next_frame().await
    }
}
