extern crate ggez;
use ggez::graphics;
use ggez::input;
use ggez::nalgebra as na;
use ggez::*;

use legion::prelude::*;

mod components;
use components::{Draw, Kinematics, Mass, Point, Position, Radius, Static, Vector};

mod main_state;
use main_state::MainState;

mod physics;

const G: f32 = 66.74;

fn main() -> GameResult {
    let (ctx, event_loop) = &mut ggez::ContextBuilder::new("N-body gravity sim", "Mikail Khan")
        .window_setup(ggez::conf::WindowSetup::default().title("Gravity"))
        .window_mode(ggez::conf::WindowMode::default().dimensions(600.0, 600.0))
        .build()
        .expect("error building context");

    let universe = Universe::new(None);
    let mut world = universe.create_world();

    world.insert_from(
        (),
        vec![
            (
                Position([500.0, 500.0].into()),
                Kinematics {
                    vel: [3.0, -3.0].into(),
                    accel: [0.0, 0.0].into(),
                    past_accel: [0.0, 0.0].into(),
                },
                Mass(0.01),
                Draw(ggez::graphics::WHITE),
                Radius(10.0),
            ),
            (
                Position([300.0, 300.0].into()),
                Kinematics {
                    vel: [0.0, 0.0].into(),
                    accel: [0.0, 0.0].into(),
                    past_accel: [0.0, 0.0].into(),
                },
                Mass(100.0),
                Draw(ggez::graphics::WHITE),
                Radius(10.0),
            ),
        ],
    );

    let main_state = &mut MainState::new(universe, world);
    event::run(ctx, event_loop, main_state)
}
