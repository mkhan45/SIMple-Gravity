use egui_macroquad::macroquad;
use macroquad::prelude::*;

pub mod error;
use error::SimError;

pub mod body_creation;
pub mod camera;
pub mod draw;
pub mod input_state;
pub mod main_state;
pub mod physics;

#[macroquad::main("Gravity")]
async fn main() -> Result<(), SimError> {
    next_frame().await;

    let mut main_state = main_state::MainState::default();

    loop {
        main_state.update()?;
        next_frame().await
    }
}
