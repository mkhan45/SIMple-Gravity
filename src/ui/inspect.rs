use bevy_ecs::prelude::*;
use egui_macroquad::egui::{self, CtxRef};
use egui_macroquad::macroquad::prelude::*;

use crate::camera::FollowBody;
use crate::physics::KinematicBody;
use crate::trails::{RelativeTrails, Trail};

use super::body_creation::CreationState;
use super::input_state::MouseState;

pub struct InspectedEntity(pub Option<Entity>);

pub fn inspect_body_sys(
    mut inspected_entity: ResMut<InspectedEntity>,
    kinematic_bodies: Query<(&KinematicBody, Entity)>,
    creation_state: Res<CreationState>,
    mouse_state: Res<MouseState>,
    egui_ctx: Res<CtxRef>,
) {
    if inspected_entity.0.is_none()
        && *creation_state != CreationState::Initiated
        && is_mouse_button_pressed(MouseButton::Left)
    {
        inspected_entity.0 = kinematic_bodies
            .iter()
            .find(|(body, _)| {
                (body.pos - mouse_state.prev_position).length_squared() < body.radius.powi(2) + 50.0
            })
            .map(|(_, e)| e);
    } else if *creation_state != CreationState::Initiated
        && (is_key_pressed(KeyCode::Escape)
            || (is_mouse_button_pressed(MouseButton::Left) && !egui_ctx.is_pointer_over_area()))
    {
        inspected_entity.0 = None;
    }
}

pub fn inspect_panel_sys(
    egui_ctx: Res<egui::CtxRef>,
    inspected_entity: Res<InspectedEntity>,
    mut followed_body: ResMut<FollowBody>,
    mut relative_trails_body: ResMut<RelativeTrails>,
    mut body_info: Query<(&mut KinematicBody, &mut Trail)>,
    mut commands: Commands,
) {
    if let Some(entity) = inspected_entity.0 {
        let (mut kinematic_body, mut trail) = match body_info.get_mut(entity) {
            Ok(b) => b,
            Err(_) => return,
        };

        egui::SidePanel::left("Inspect").show(&egui_ctx, |ui| {
            ui.spacing_mut().slider_width = 200.0;

            ui.add(
                egui::Slider::new(&mut kinematic_body.radius, 0.0..=1_000.0)
                    .text("Radius")
                    .logarithmic(true),
            );

            ui.add(
                egui::Slider::new(&mut kinematic_body.mass, 0.0..=10_000.0)
                    .text("Mass")
                    .logarithmic(true),
            );

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

            ui.add(egui::Slider::new(&mut trail.max_len, 0..=10_000).text("Trail Max Length"));

            if ui.button("Follow").clicked() {
                if followed_body.0.contains(&entity) {
                    *followed_body = FollowBody(None)
                } else {
                    *followed_body = FollowBody(Some(entity));
                }
            }
            if ui.button("RelativeTrails").clicked() {
                if relative_trails_body.0.contains(&entity) {
                    *relative_trails_body = RelativeTrails(None);
                } else {
                    *relative_trails_body = RelativeTrails(Some(entity));
                }
            }
            if ui.button("Delete").clicked() {
                commands.entity(entity).despawn();
            }
        });
    }
}
