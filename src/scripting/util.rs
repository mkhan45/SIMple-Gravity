use crate::physics::KinematicBody;
use egui_macroquad::macroquad::prelude::*;

macro_rules! gen_accessors {
    ($get_ident:ident, $set_ident:ident, $field:ident, $ty:ty) => {
        pub fn $get_ident(&mut self) -> $ty {
            self.$field.clone()
        }

        pub fn $set_ident(&mut self, new_val: $ty) {
            self.$field = new_val
        }
    };
}

impl KinematicBody {
    pub fn from_rhai(body: rhai::Map) -> Self {
        let pos = body
            .get("pos")
            .and_then(|pos| pos.clone().try_cast::<Vec2>())
            .unwrap_or(Vec2::new(0.0, 0.0));

        let vel = body
            .get("vel")
            .and_then(|vel| vel.clone().try_cast::<Vec2>())
            .unwrap_or(Vec2::new(0.0, 0.0));

        let accel = body
            .get("accel")
            .and_then(|accel| accel.clone().try_cast::<Vec2>())
            .unwrap_or(Vec2::new(0.0, 0.0));

        let force = body
            .get("force")
            .and_then(|force| force.clone().try_cast::<Vec2>())
            .unwrap_or(Vec2::new(0.0, 0.0));

        let mass = body
            .get("mass")
            .and_then(|mass| mass.clone().try_cast::<f32>())
            .unwrap_or(10.0);

        let radius = body
            .get("radius")
            .and_then(|radius| radius.clone().try_cast::<f32>())
            .unwrap_or(10.0);

        KinematicBody {
            pos,
            vel,
            accel,
            force,
            mass,
            radius,
        }
    }

    gen_accessors!(get_pos, set_pos, pos, Vec2);
    gen_accessors!(get_vel, set_vel, vel, Vec2);
    gen_accessors!(get_force, set_force, force, Vec2);
    gen_accessors!(get_accel, set_accel, accel, Vec2);
    gen_accessors!(get_mass, set_mass, mass, f32);
    gen_accessors!(get_radius, set_radius, radius, f32);
}
