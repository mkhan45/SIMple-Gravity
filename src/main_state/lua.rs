use crate::ecs::components::{Kinematics, Point, Position, Vector};
use crate::ecs::entities::create_body;
use crate::ecs::resources::LuaRes;
use crate::main_state::state::MainState;

use specs::prelude::*;

impl MainState<'_, '_> {
    #[allow(clippy::many_single_char_names)]
    pub fn process_lua_body(&mut self, body: &rlua::Table) {
        {
            let entities = self.world.entities();
            let num_bodies = entities.join().count();
            if num_bodies > 1250 {
                println!("Can't have more than 1250 bodies");
                return;
            }
        }
        let mass: f32 = body.get("mass").unwrap();
        let x: f32 = body.get("x").unwrap();
        let y: f32 = body.get("y").unwrap();
        let x_vel = body.get("x_vel").unwrap_or(0.0);
        let y_vel = body.get("y_vel").unwrap_or(0.0);
        let rad: f32 = body.get("rad").unwrap();

        let color = body
            .get::<_, rlua::Table>("color")
            .map(|color_table| {
                match (
                    color_table.get("r"),
                    color_table.get("g"),
                    color_table.get("b"),
                    color_table.get("a"),
                ) {
                    (Ok(r), Ok(g), Ok(b), Ok(a)) => {
                        Some(ggez::graphics::Color::from_rgba(r, g, b, a))
                    }
                    _ => None,
                }
                .unwrap_or(ggez::graphics::WHITE)
            })
            .unwrap_or(ggez::graphics::WHITE);

        create_body(
            &mut self.world,
            (
                Position(Point::new(x, y)),
                Kinematics::new(Vector::new(x_vel, y_vel)),
                crate::ecs::components::Mass(mass),
                crate::ecs::components::Draw(color),
                crate::ecs::components::Radius(rad),
                crate::ecs::components::Trail::new(35),
            ),
        );
    }

    pub fn process_lua_bodies(&mut self) {
        let lua = self.world.fetch_mut::<LuaRes>().clone();

        lua.lock().unwrap().context(|lua_ctx| {
            let globals = lua_ctx.globals();
            let bodies: Vec<rlua::Table> = globals.get("BODIES").unwrap();
            bodies.iter().for_each(|body| self.process_lua_body(body));
        });
    }

    pub fn init_lua(&mut self) {
        let lua = self.world.fetch_mut::<LuaRes>().clone();

        lua.lock().unwrap().context(|lua_ctx| {
            let globals = lua_ctx.globals();
            let bodies: Vec<rlua::Table> = Vec::new();
            globals.set("BODIES", bodies).unwrap();

            lua_ctx
                .load(
                    r#"
                function add_body(body)
                    BODIES[#BODIES + 1] = body
                end

                function add_bodies(...)
                    for _, body in ipairs({...}) do
                        add_body(body)
                    end
                end
            "#,
                )
                .exec()
                .unwrap();
        });

        let _ = crate::saveload::load_from_lua(&self.world, "saved_systems/default.lua".to_string());
    }

    pub fn lua_update(&mut self) {
        self.process_lua_bodies();
        let lua = self.world.fetch_mut::<LuaRes>().clone();

        lua.lock().unwrap().context(|lua_ctx| {
            let globals = lua_ctx.globals();
            let bodies: Vec<rlua::Table> = Vec::new();
            globals.set("BODIES", bodies).unwrap();
        });
    }
}
