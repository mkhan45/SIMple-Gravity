use legion::prelude::*;

use ggez::{
    event::EventHandler,
    graphics,
    graphics::{DrawMode, DrawParam, Color},
    input,
    input::{
        keyboard::{KeyCode, KeyMods},
        mouse::MouseButton,
    },
    Context, GameResult,
};

use crate::{Draw, Point, Position, Vector, Radius};

pub struct MainState {
    universe: Universe,
    main_world: World,
}

impl MainState {
    pub fn new(universe: Universe, main_world: World) -> Self {
        MainState { universe, main_world}
    }
}

impl EventHandler for MainState {
    fn update(&mut self, ctx: &mut Context) -> GameResult {
        Ok(())
    }

    fn draw(&mut self, ctx: &mut Context) -> GameResult {
        graphics::clear(ctx, Color::new(0.0, 0.0, 0.0, 1.0));
        let draw_query = <(Read<Draw>, Read<Position>, Read<Radius>)>::query();
        draw_query.iter(&self.main_world).for_each(|(color, pos, rad)| {
            let point: ggez::mint::Point2<f32> = (*pos).into();
            let circle = ggez::graphics::Mesh::new_circle(ctx, DrawMode::fill(), point, rad.0, 0.05, color.0).expect("error building mesh");
            ggez::graphics::draw(ctx, &circle, graphics::DrawParam::new()).expect("error drawing mesh");
        });
        ggez::graphics::present(ctx)
    }
}
