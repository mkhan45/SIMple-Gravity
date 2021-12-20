use bevy_ecs::prelude::*;
use egui_macroquad::macroquad::prelude::*;

use crate::physics::KinematicBody;

pub struct MainState {
    pub world: World,
    main_physics_schedule: Schedule,
    preview_physics_schedule: Schedule,
    draw_schedule: Schedule,
}

impl Default for MainState {
    fn default() -> Self {
        let world = {
            let mut world = World::new();

            world.insert_resource(crate::physics::DT(1.0));

            world.spawn().insert(KinematicBody {
                pos: Vec2::new(500.0, 500.0),
                mass: 10.0,
                radius: 50.0,
                ..Default::default()
            });

            world.spawn().insert(KinematicBody {
                pos: Vec2::new(15.0, 15.0),
                mass: 0.1,
                radius: 10.0,
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
                    .with_system(crate::physics::integration_sys.system().after("gravity")),
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
                SystemStage::single_threaded().with_system(crate::draw::draw_bodies_sys.system()),
            );

            draw_schedule
        };

        Self {
            world,
            main_physics_schedule,
            preview_physics_schedule,
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

        self.draw_schedule.run(&mut self.world);

        Ok(())
    }
}
