use bevy_ecs::prelude::*;
use egui_macroquad::macroquad::prelude::*;

use crate::physics::{KinematicBody, Paused};

use crate::ui::body_creation::{CreationData, CreationState};
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
        let world = {
            let mut world = World::new();

            world.insert_resource(crate::physics::DT(1.0));

            let camera_res = crate::camera::CameraRes::default();
            set_camera(&camera_res.camera);
            world.insert_resource(camera_res);

            let mouse_state_res = MouseState::default();
            world.insert_resource(mouse_state_res);

            world.insert_resource(CreationData::default());
            world.insert_resource(CreationState::Unstarted);
            world.insert_resource(InspectedEntity(None));
            world.insert_resource(egui_macroquad::egui::CtxRef::default());

            world.insert_resource(Paused(false));

            world.spawn().insert(KinematicBody {
                pos: Vec2::new(0.0, 0.0),
                mass: 2500.0,
                radius: 150.0,
                ..Default::default()
            });

            world.spawn().insert(KinematicBody {
                pos: Vec2::new(3500.0, 0.0),
                mass: 0.01,
                radius: 25.0,
                vel: Vec2::new(0.0, -7.5),
                ..Default::default()
            });

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
                    .with_system(crate::physics::integration_sys.system().after("collision")),
            );

            main_physics_schedule
        };

        let preview_physics_schedule = {
            let preview_physics_schedule = Schedule::default();
            preview_physics_schedule
        };

        let draw_schedule = {
            let mut draw_schedule = Schedule::default();

            draw_schedule.add_stage(
                "draw",
                SystemStage::single_threaded()
                    .with_system(crate::draw::draw_bodies_sys.system())
                    .with_system(crate::draw::draw_create_preview.system())
                    .with_system(crate::camera::update_camera_sys.system())
                    .with_system(crate::camera::camera_transform_sys.system()),
            );

            draw_schedule.add_stage(
                "gui",
                SystemStage::single_threaded()
                    .with_system(crate::ui::initialize_gui_sys.system().label("initialize"))
                    .with_system(
                        crate::ui::top_panel::top_panel_sys
                            .system()
                            .label("top_panel")
                            .after("initialize"),
                    )
                    .with_system(
                        crate::ui::inspect::inspect_panel_sys
                            .system()
                            .after("top_panel"),
                    ),
            );

            draw_schedule
        };

        let input_schedule = {
            let mut input_schedule = Schedule::default();

            input_schedule.add_stage(
                "update_input",
                SystemStage::single_threaded()
                    .with_system(crate::ui::input_state::update_mouse_input_sys.system())
                    .with_system(crate::ui::body_creation::create_body_sys.system())
                    .with_system(crate::ui::inspect::inspect_body_sys.system()),
            );

            input_schedule
        };

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

        for _ in 0..10 {
            self.preview_physics_schedule.run(&mut self.world);
        }

        clear_background(BLACK);
        self.draw_schedule.run(&mut self.world);
        egui_macroquad::draw();
        self.input_schedule.run(&mut self.world);

        Ok(())
    }
}
