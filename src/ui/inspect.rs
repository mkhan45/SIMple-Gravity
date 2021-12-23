use bevy_ecs::prelude::*;
use egui_macroquad::egui;
use egui_macroquad::macroquad::prelude::*;

use crate::physics::KinematicBody;

use super::input_state::MouseState;

pub struct InspectedEntity(pub Option<Entity>);

pub fn inspect_body_sys(
    mut inspected_entity: ResMut<InspectedEntity>,
    kinematic_bodies: Query<(&KinematicBody, Entity)>,
    mouse_state: Res<MouseState>,
) {
    if inspected_entity.0.is_none() && is_mouse_button_pressed(MouseButton::Left) {
        inspected_entity.0 = kinematic_bodies
            .iter()
            .find(|(body, _)| {
                (body.pos - mouse_state.prev_position).length_squared() < body.radius.powi(2)
            })
            .map(|(_, e)| e);
    }
}

pub fn inspect_panel_sys(
    egui_ctx: Res<egui::CtxRef>,
    inspected_entity: Res<InspectedEntity>,
    mut kinematic_bodies: Query<&mut KinematicBody>,
) {
    if let Some(entity) = inspected_entity.0 {
        let mut kinematic_body = kinematic_bodies.get_mut(entity).unwrap();

        egui::SidePanel::left("Inspect").show(&egui_ctx, |ui| {
            ui.spacing_mut().slider_width = 200.0;

            ui.add(egui::Slider::new(&mut kinematic_body.radius, 0.0..=1_000.0).text("Radius"));

            ui.add(egui::Slider::new(&mut kinematic_body.mass, 0.0..=10_000.0).text("Mass"));

            ui.label(format!(
                "Position: <{:.2}, {:.2}>",
                kinematic_body.pos.x, kinematic_body.pos.y
            ));
            ui.label(format!(
                "Velocity: <{:.2}, {:.2}>",
                kinematic_body.vel.x, kinematic_body.vel.y
            ));
            ui.label(format!(
                "Acceleration: <{:.2}, {:.2}>",
                kinematic_body.accel.x, kinematic_body.accel.y
            ));
        });
    }
}
