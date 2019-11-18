extern crate ggez;
use ggez::*;

use legion::prelude::*;

mod components;
use components::{Draw, Kinematics, Mass, Point, Position, Radius, Vector};

mod main_state;
use main_state::MainState;

mod physics;
mod imgui_wrapper;
use imgui_wrapper::ImGuiWrapper;

const G: f32 = 66.74;
type Body = (Position, Kinematics, Mass, Draw, Radius);

pub fn new_body(pos: impl Into<Point>, vel: impl Into<Vector>, mass: f32, rad: f32) -> Body {
    (
        Position(pos.into()),
        Kinematics::new(vel.into()),
        Mass(mass),
        Draw(ggez::graphics::WHITE),
        Radius(rad),
    )
}

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
                new_body([500.0, 500.0], [3.0, -3.0], 0.01, 5.0),
                new_body([300.0, 300.0], [0.0, 0.0], 100.0, 20.0),
                new_body([100.0, 100.0], [-1.6, -0.5], 1.0, 10.0),
            ],
        );

    // world.insert_from(
    //     (),
    //     (0..1100).map(|i| {
    //         (new_body(
    //             [(i / 10) as f32 * 100.0, (i % 10) as f32 * 100.0],
    //             [0.0, 0.0],
    //             -0.1,
    //             5.0,
    //         ))
    //     }),
    // );
    let hidpi_factor = event_loop.get_primary_monitor().get_hidpi_factor() as f32;

    let main_state = &mut MainState::new(universe, world, ImGuiWrapper::new(ctx, hidpi_factor), hidpi_factor);
    event::run(ctx, event_loop, main_state)
}
