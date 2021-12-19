use egui_macroquad::macroquad;
use macroquad::prelude::*;

pub mod error;
use error::SimError;

pub mod physics;

#[macroquad::main("Gravity")]
async fn main() -> Result<(), SimError> {
    next_frame().await;

    loop {
        clear_background(BLACK);
        next_frame().await
    }
}
