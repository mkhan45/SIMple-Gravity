use legion::prelude::*;

use ggez::{
    event::EventHandler,
    graphics,
    graphics::{Color, DrawMode, DrawParam},
    input,
    input::{
        keyboard::{KeyCode, KeyMods},
        mouse::MouseButton,
    },
    Context, GameResult,
};

use crate::{Draw, Kinematics, Mass, Point, Position, Radius, Static, Vector, G};

pub const DT: f32 = 0.5;

pub struct MainState {
    universe: Universe,
    main_world: World,
}

impl MainState {
    pub fn new(universe: Universe, main_world: World) -> Self {
        MainState {
            universe,
            main_world,
        }
    }
}

impl EventHandler for MainState {
    fn update(&mut self, ctx: &mut Context) -> GameResult {
        let gravity_query = <(Read<Position>, Write<Kinematics>, Read<Radius>)>::query();
        let inner_query = <(Read<Position>, Read<Mass>, Read<Radius>)>::query();

        gravity_query.iter(&self.main_world).for_each(|(current_pos, kinematics, rad1)| {

            kinematics.accel = [0.0, 0.0].into();

            inner_query.iter(&self.main_world).for_each(|(other_pos, other_mass, rad2)| {
                let dist_vec = other_pos.0 - current_pos.0;
                let dist_mag = current_pos.dist(&other_pos);

                if dist_mag >= rad1.0 + rad2.0 {
                    let dist_comp = dist_vec / dist_mag;

                    let grav_accel_mag = other_mass.0 / dist_mag.powi(2) * G;
                    let grav_accel: Vector = dist_comp * grav_accel_mag;

                    kinematics.accel += grav_accel
                }
            });

        });

        // let integrate_query = <(Write<Position>, Write<Kinematics>)>::query().filter(!shared_data::<Static>()); // doesn't work idk why
        let integrate_query = <(Write<Position>, Write<Kinematics>)>::query();
        integrate_query
            .iter(&self.main_world)
            .for_each(|(pos, kinematics)| {
                let vel = &mut kinematics.vel;
                let accel = kinematics.accel;

                vel.x += accel.x * DT;
                vel.y += accel.y * DT;

                pos.0.x += vel.x * DT + accel.x / 2.0 * DT.powi(2);
                pos.0.y += vel.y * DT + accel.y / 2.0 * DT.powi(2);
            });

        Ok(())
    }

    fn draw(&mut self, ctx: &mut Context) -> GameResult {
        graphics::clear(ctx, Color::new(0.0, 0.0, 0.0, 1.0));
        let draw_query = <(Read<Draw>, Read<Position>, Read<Radius>)>::query();
        draw_query
            .iter(&self.main_world)
            .for_each(|(color, pos, rad)| {
                let point: ggez::mint::Point2<f32> = (*pos).into();
                let circle = ggez::graphics::Mesh::new_circle(
                    ctx,
                    DrawMode::fill(),
                    point,
                    rad.0,
                    0.05,
                    color.0,
                )
                .expect("error building mesh");
                ggez::graphics::draw(ctx, &circle, graphics::DrawParam::new())
                    .expect("error drawing mesh");
            });
        ggez::graphics::present(ctx)
    }
}
