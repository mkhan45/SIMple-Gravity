extern crate ggez;
use ggez::*; use ggez::graphics; use ggez::nalgebra as na;
use ggez::input;

use legion::prelude::*;

mod components;
use components::{Kinematics, Point, Vector};

mod main_state;
use main_state::MainState;

const G: f32 = 6.674;

fn main() -> GameResult {
    let (ctx, event_loop) = &mut ggez::ContextBuilder::new("N-body gravity sim", "Mikail Khan")
        .window_setup(ggez::conf::WindowSetup::default().title("Gravity"))
        .window_mode(ggez::conf::WindowMode::default().dimensions(600.0, 600.0))
        .build().expect("error building context");

    let universe = Universe::new(None);
    let mut world = universe.create_world();

    // world.insert_from(
    //     (),
    //     vec![
    //         (Kinematics::new(400.0, 400.0)),
    //     ],
    // );

    let main_state = &mut MainState::new(universe);
    event::run(ctx, event_loop, main_state)
}
