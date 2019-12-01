extern crate ggez;
use ggez::*;

use legion::prelude::*;

mod components;
use components::{Draw, Kinematics, Mass, Point, Position, Preview, Radius, Trail, Vector};

mod main_state;
use main_state::MainState;

mod imgui_wrapper;
mod physics;
use imgui_wrapper::ImGuiWrapper;

const G: f32 = 1.2;
const SCREEN_X: f32 = 300.0;
const SCREEN_Y: f32 = 300.0;

type Body = (Position, Kinematics, Mass, Draw, Radius, Trail);
type PreviewBody = (Position, Kinematics, Radius, Preview, Draw);

pub fn new_body(pos: impl Into<Point>, vel: impl Into<Vector>, mass: f32, rad: f32) -> Body {
    (
        Position(pos.into()),
        Kinematics::new(vel.into()),
        Mass(mass),
        Draw(ggez::graphics::WHITE),
        Radius(rad),
        Trail(Vec::with_capacity(250)),
    )
}

pub fn new_preview(pos: impl Into<Point>, vel: impl Into<Vector>, rad: f32) -> PreviewBody {
    (
        Position(pos.into()),
        Kinematics::new(vel.into()),
        Radius(rad),
        Preview,
        Draw(graphics::Color::new(0.1, 1.0, 0.2, 0.8)),
    )
}

fn main() -> GameResult {
    let (ctx, event_loop) = &mut ggez::ContextBuilder::new("N-body gravity sim", "Mikail Khan")
        .window_setup(ggez::conf::WindowSetup::default().title("Gravity"))
        .window_mode(ggez::conf::WindowMode::default().dimensions(600.0, 600.0))
        .build()
        .expect("error building context");

    let universe = Universe::new();
    let mut world = universe.create_world();

    let data = vec![
        new_body([215.0, 100.0], [-0.0, -1.1], 0.01, 0.8),
        new_body([150.0, 100.0], [0.0, 0.0], 75.0, 5.0),
    ];

    world.insert(
        (),
        data,
    );

    // world.insert(
    //     (),
    //     (0..1000).map(|i| {
    //         (new_body(
    //             [(i / 10) as f32 * 100.0, (i % 10) as f32 * 100.0],
    //             [0.0, 0.0],
    //             -0.1,
    //             5.0,
    //         ))
    //     }),
    // );
    let hidpi_factor = event_loop.get_primary_monitor().get_hidpi_factor() as f32;
    let dimensions = event_loop.get_primary_monitor().get_dimensions();
    let dimensions_vec = Vector::new(dimensions.width as f32, dimensions.height as f32);
    let aspect_ratio = dimensions.height / dimensions.width;
    graphics::set_mode(
        ctx,
        ggez::conf::WindowMode::default()
            .resizable(true)
            .fullscreen_type(ggez::conf::FullscreenType::Desktop),
    )
    .expect("error resizing window");

    graphics::set_screen_coordinates(
        ctx,
        graphics::Rect::new(0., 0., SCREEN_X, SCREEN_Y * aspect_ratio as f32),
    )
    .unwrap();

    let main_state = &mut MainState::new(
        universe,
        world,
        ImGuiWrapper::new(ctx, hidpi_factor, dimensions_vec),
        hidpi_factor,
        dimensions_vec,
    );
    event::run(ctx, event_loop, main_state)
}
