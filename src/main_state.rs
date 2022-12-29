use bevy_ecs::prelude::*;
use egui_macroquad::egui::FontFamily;
use egui_macroquad::macroquad::prelude::*;

use crate::physics::{self, Paused};

use crate::scripting::RhaiRes;
use crate::ui::body_creation::{CreationData, CreationState};
use crate::ui::code_editor::CodeEditor;
use crate::ui::input_state::MouseState;
use crate::ui::inspect::InspectedEntity;

pub struct MainState {
    pub world: World,
    main_physics_schedule: Schedule,
    preview_physics_schedule: Schedule,
    input_schedule: Schedule,
    draw_schedule: Schedule,
}

impl Default for MainState {
    fn default() -> Self {
        let mut world = {
            let mut world = World::new();

            world.insert_resource(crate::physics::DT(1.0));
            world.insert_resource(crate::physics::G(100.0));

            let camera_res = crate::camera::CameraRes::default();
            set_camera(&camera_res.camera);
            world.insert_resource(camera_res);
            world.insert_resource(crate::camera::FollowBody(None));
            world.insert_resource(crate::trails::RelativeTrails(None));

            let mouse_state_res = MouseState::default();
            world.insert_resource(mouse_state_res);

            world.insert_resource(CreationData::default());
            world.insert_resource(CreationState::Unstarted);
            world.insert_resource(InspectedEntity(None));
            world.insert_resource(egui_macroquad::egui::Context::default());

            world.insert_resource(Paused(false));
            world.insert_resource(physics::PhysicsToggles { collisions: true, integration: true });

            world.insert_resource(crate::preview::PreviewTrailTick::default());
            world.insert_resource(crate::preview::MultiPreview(false));

            world.insert_resource(crate::force_lines::DrawForceLines(false));
            world.insert_resource(crate::trails::DrawTrails(true));

            world.insert_resource(crate::ui::code_editor::CodeEditor::default());

            world.insert_resource(RhaiRes::default());

            world
        };

        let main_physics_schedule = {
            let mut main_physics_schedule = Schedule::default();

            main_physics_schedule.add_stage(
                "physics",
                SystemStage::single_threaded()
                    .with_system(crate::physics::gravity_sys.system().label("gravity"))
                    .with_system(
                        crate::physics::collision_sys
                            .system()
                            .label("collision")
                            .after("gravity"),
                    )
                    .with_system(crate::physics::integration_sys.system().after("collision"))
                    .with_system(crate::trails::trail_sys.system())
                    .with_system(crate::trails::clear_trails_sys.system())
                    .with_system(crate::force_lines::force_line_sys.system()),
            );

            main_physics_schedule
        };

        let preview_physics_schedule = {
            let mut preview_physics_schedule = Schedule::default();

            preview_physics_schedule.add_stage(
                "preview",
                SystemStage::single_threaded()
                    .with_system(
                        crate::preview::preview_gravity_sys
                            .system()
                            .label("gravity"),
                    )
                    .with_system(
                        crate::physics::preview_integration_sys
                            .system()
                            .label("integration")
                            .after("gravity"),
                    )
                    .with_system(
                        crate::trails::preview_trail_sys
                            .system()
                            .after("integration"),
                    ),
            );

            preview_physics_schedule
        };

        let draw_schedule = {
            let mut draw_schedule = Schedule::default();

            draw_schedule.add_stage(
                "draw",
                SystemStage::single_threaded()
                    .with_system(crate::draw::draw_bodies_sys.system().label("bodies"))
                    .with_system(crate::draw::draw_create_preview.system())
                    .with_system(crate::trails::draw_trail_sys.system().before("bodies"))
                    .with_system(crate::draw::draw_force_lines.system().before("bodies"))
                    .with_system(crate::camera::update_camera_sys.system())
                    .with_system(
                        crate::camera::camera_transform_sys
                            .system()
                            .label("transform"),
                    )
                    .with_system(crate::camera::camera_follow_sys.system().after("transform")),
            );

            draw_schedule.add_stage(
                "gui",
                SystemStage::single_threaded()
                    .with_system(
                        crate::ui::top_panel::top_panel_sys
                            .system()
                            .label("top_panel"),
                    )
                    .with_system(
                        crate::ui::inspect::inspect_panel_sys
                            .system()
                            .after("top_panel"),
                    )
                    .with_system(
                        crate::ui::code_editor::code_editor_sys
                            .system()
                            .after("top_panel"),
                    ),
            );

            draw_schedule
        };

        let mut input_schedule = {
            let mut input_schedule = Schedule::default();

            input_schedule.add_stage(
                "update_input",
                SystemStage::single_threaded()
                    .with_system(
                        crate::ui::input_state::update_mouse_input_sys
                            .system()
                            .label("update_mouse"),
                    )
                    .with_system(
                        crate::ui::inspect::inspect_body_sys
                            .system()
                            .label("inspect"),
                    )
                    .with_system(
                        crate::ui::body_creation::create_body_sys
                            .system()
                            .after("inspect")
                            .before("update_mouse"),
                    )
                    .with_system(crate::ui::handle_keybinds_sys.system()),
            );

            input_schedule.add_stage(
                "scripting",
                SystemStage::single_threaded()
                    .with_system(crate::scripting::run_code_sys.system().label("run"))
                    .with_system(
                        crate::scripting::run_rhai_commands_sys
                            .system()
                            .label("commands")
                            .after("run"),
                    )
                    .with_system(
                        crate::scripting::run_script_update_sys
                            .system()
                            .label("update")
                            .after("commands"),
                    ),
            );

            input_schedule
        };

        {
            use std::sync::{Arc, RwLock};
            world.insert_resource(CodeEditor {
                code: Arc::new(RwLock::new(include_str!("../rhai_scripts/basic_orbit.rhai").to_string())),
                should_run: true,
                ..CodeEditor::default()
            });
            input_schedule.run(&mut world);
            world.insert_resource(CodeEditor::default());
        }

        Self {
            world,
            main_physics_schedule,
            preview_physics_schedule,
            input_schedule,
            draw_schedule,
        }
    }
}

impl MainState {
    pub fn update(&mut self) -> Result<(), crate::error::SimError> {
        self.main_physics_schedule.run(&mut self.world);

        let was_paused = self.world.get_resource::<Paused>().unwrap().0;
        self.world.insert_resource(Paused(false));
        let start_time = get_time();
        while get_time() - start_time < 0.0075 {
            self.preview_physics_schedule.run(&mut self.world);
        }
        self.world.insert_resource(Paused(was_paused));

        clear_background(BLACK);

        egui_macroquad::ui(|egui_ctx| {
            use egui_macroquad::egui::{FontId, TextStyle};
            let mut style = (*egui_ctx.style()).clone();
            style.text_styles.insert(
                TextStyle::Button,
                FontId::new(28.0, FontFamily::Proportional),
            );
            style
                .text_styles
                .insert(TextStyle::Body, FontId::new(28.0, FontFamily::Proportional));
            style.text_styles.insert(
                TextStyle::Monospace,
                FontId::new(24.0, FontFamily::Monospace),
            );
            egui_ctx.set_style(style);
            // let mut fonts = FontDefinitions::default();
            // fonts.family_and_size.get_mut(&TextStyle::Button).unwrap().1 = 28.0;
            // fonts.family_and_size.get_mut(&TextStyle::Body).unwrap().1 = 28.0;
            // fonts
            //     .family_and_size
            //     .get_mut(&TextStyle::Monospace)
            //     .unwrap()
            //     .1 = 24.0;
            // egui_ctx.set_fonts(fonts);

            self.world.insert_resource(egui_ctx.clone());
            self.draw_schedule.run(&mut self.world);
        });
        egui_macroquad::draw();

        self.input_schedule.run(&mut self.world);

        Ok(())
    }
}
