use bevy_ecs::prelude::*;
use egui_macroquad::macroquad::prelude::*;
use rhai::{Engine, Scope};

use crate::{
    physics::{KinematicBody, PhysicsToggles, G, Paused, DT},
    ui::code_editor::CodeEditor,
};

use slotmap::{DefaultKey, SlotMap, KeyData};

use std::{
    collections::BTreeMap,
    sync::{Arc, RwLock},
};

pub mod samples;
mod util;

pub struct RhaiID(pub DefaultKey);

pub enum RhaiCommand {
    UpdateBody { id: DefaultKey, params: rhai::Map }, // TODO: set timestep, add graph, etc.
    DeleteBody { id: DefaultKey },
    SetG(f32),
    SetCollisions(bool),
    SetIntegration(bool),
    Draw { params: rhai::Map },
    Export,
    SetPaused(bool),
}

#[derive(Clone)]
pub enum DrawFn {
    Draw(Arc<dyn Fn() -> DrawFn + Send + Sync>),
    Finished,
}
pub struct RhaiBody;
pub struct RhaiRes {
    pub engine: Engine,
    pub scope: Scope<'static>,
    pub output: Arc<RwLock<String>>,
    pub newly_added_bodies: Arc<RwLock<SlotMap<DefaultKey, rhai::Map>>>,
    pub existing_bodies: Arc<RwLock<BTreeMap<DefaultKey, Entity>>>,
    pub names: BTreeMap<String, DefaultKey>,
    pub commands: Arc<RwLock<Vec<RhaiCommand>>>,
    pub drawings: DrawFn,
    pub last_code: rhai::AST,
    pub lib_ast: rhai::AST,
}

impl Default for RhaiRes {
    fn default() -> Self {
        let mut scope = Scope::new();
        scope.push("_console_output", String::new());

        let mut engine = Engine::new();
        engine.set_max_expr_depths(0, 0);

        let output = Arc::new(RwLock::new(String::new()));

        let logger = output.clone();
        engine.on_print(move |s| logger.write().unwrap().push_str(&format!("{}\n", s)));

        engine
            .register_type::<KinematicBody>()
            .register_get_set("pos", KinematicBody::get_pos, KinematicBody::set_pos)
            .register_get_set("vel", KinematicBody::get_vel, KinematicBody::set_vel)
            .register_get_set("accel", KinematicBody::get_accel, KinematicBody::set_accel)
            .register_get_set("force", KinematicBody::get_force, KinematicBody::set_force)
            .register_get_set("mass", KinematicBody::get_mass, KinematicBody::set_mass)
            .register_get_set(
                "radius",
                KinematicBody::get_radius,
                KinematicBody::set_radius,
            );

        engine
            .register_type::<Vec2>()
            .register_get_set("x", |v: &mut Vec2| v.x, |v: &mut Vec2, val: f32| v.x = val)
            .register_get_set("y", |v: &mut Vec2| v.y, |v: &mut Vec2, val: f32| v.y = val)
            .register_get("length", |v: &mut Vec2| v.length())
            .register_get("normalized", |v: &mut Vec2| v.normalize());

        engine.register_fn("id", move |id_num: i64| {
            DefaultKey::from(KeyData::from_ffi(id_num as u64))
        });
        engine.register_fn("to_string", |v: Vec2| v.to_string());
        engine.register_fn("to_debug", |v: Vec2| format!("{:?}", v));
        engine.register_fn("angle_between", |a: Vec2, b: Vec2| a.angle_between(b));
        engine.register_fn("rotate", |a: Vec2, b: f32| Vec2::from_angle(b).rotate(a));

        engine.register_fn("-", |v: Vec2| -v);
        engine.register_fn("-", |lhs: Vec2, rhs: Vec2| lhs - rhs);
        engine.register_fn("+", |lhs: Vec2, rhs: Vec2| lhs + rhs);

        engine.register_fn("/", |lhs: Vec2, rhs: f32| lhs / rhs);
        engine.register_fn("/", |lhs: f32, rhs: Vec2| lhs / rhs);

        engine.register_fn("/", |lhs: Vec2, rhs: i64| lhs / rhs as f32);
        engine.register_fn("/", |lhs: i64, rhs: Vec2| lhs as f32 / rhs);

        engine.register_fn("*", |lhs: Vec2, rhs: f32| lhs * rhs);
        engine.register_fn("*", |lhs: f32, rhs: Vec2| lhs * rhs);

        engine.register_fn("*", |lhs: Vec2, rhs: i64| lhs * rhs as f32);
        engine.register_fn("*", |lhs: i64, rhs: Vec2| lhs as f32 * rhs);

        engine.register_type::<Entity>();
        engine.register_type::<DefaultKey>();
        engine.register_type::<SlotMap<DefaultKey, KinematicBody>>();
        engine
            .register_type::<Vec<DefaultKey>>()
            .register_get("length", |v: &mut Vec<DefaultKey>| v.len() as i64);
        engine.register_iterator::<Vec<DefaultKey>>();
        engine.register_indexer_get(|v: &mut Vec<DefaultKey>, ix: i64| v[ix as usize]);

        engine.register_fn("insert", SlotMap::<DefaultKey, KinematicBody>::insert);
        engine.register_fn("get", SlotMap::<DefaultKey, KinematicBody>::get);

        engine.register_fn("vec", Vec2::new);
        engine.register_fn("vec", |x: i64, y: i64| Vec2::new(x as f32, y as f32));
        engine.register_fn("new_body", KinematicBody::from_rhai);

        let newly_added_bodies = Arc::new(RwLock::new(SlotMap::<DefaultKey, rhai::Map>::new()));
        let existing_bodies = Arc::new(RwLock::new(BTreeMap::<DefaultKey, Entity>::new()));

        let new_bodies_ref = newly_added_bodies.clone();
        engine.register_fn("add_body", move |body| {
            new_bodies_ref.write().unwrap().insert(body)
        });

        let commands = Arc::new(RwLock::new(Vec::new()));

        let command_ref = commands.clone();
        engine.register_fn("update_body", move |id, params| {
            let mut commands_writer = command_ref.write().unwrap();
            commands_writer.push(RhaiCommand::UpdateBody { id, params });
        });

        let command_ref = commands.clone();
        engine.register_fn("delete_body", move |id| {
            let mut commands_writer = command_ref.write().unwrap();
            commands_writer.push(RhaiCommand::DeleteBody { id });
        });

        let command_ref = commands.clone();
        engine.register_fn("set_g", move |new_g| {
            let mut commands_writer = command_ref.write().unwrap();
            commands_writer.push(RhaiCommand::SetG(new_g));
        });

        let command_ref = commands.clone();
        engine.register_fn("set_collisions", move |enabled| {
            let mut commands_writer = command_ref.write().unwrap();
            commands_writer.push(RhaiCommand::SetCollisions(enabled));
        });

        let command_ref = commands.clone();
        engine.register_fn("set_integration", move |enabled| {
            let mut commands_writer = command_ref.write().unwrap();
            commands_writer.push(RhaiCommand::SetIntegration(enabled));
        });

        let command_ref = commands.clone();
        engine.register_fn("set_paused", move |enabled| {
            let mut commands_writer = command_ref.write().unwrap();
            commands_writer.push(RhaiCommand::SetPaused(enabled));
        });

        let command_ref = commands.clone();
        engine.register_fn("draw", move |params| {
            let mut commands_writer = command_ref.write().unwrap();
            commands_writer.push(RhaiCommand::Draw { params });
        });

        // TODO: Constify via a macro
        let mut lib_code = "
            fn reset_physics() {
                set_g(100.0);
                set_collisions(true);
                set_integration(true);
            }
        "
        .to_string();
        for field in ["pos", "vel", "accel", "force", "mass", "radius"] {
            // the set_{field} functions are weird because of the 1-frame
            // delay and black magic
            lib_code.push_str(&format!(
                "
                fn get_{0}(body) {{ get_body(body).{0} }}
                fn get_{0}() {{ get_body(this).{0} }}

                fn add_{0}(body, field) {{ update_body(body, #{{ add_{0}: field }}) }}
                fn add_{0}(field) {{ update_body(this, #{{ add_{0}: field }} ) }}

                fn set_{0}(body, field) {{ update_body(body, #{{ add_{0}: -body.get_{0}() + field }}) }}
                fn set_{0}(field) {{ update_body(this, #{{ add_{0}: -this.get_{0}() + field }} ) }}
            ",
                field
            ));
        }

        lib_code.push_str("
            fn draw_line(start, end, thickness) {
                draw(#{
                    type: \"line\",
                    start: start,
                    end: end,
                    thickness: thickness,
                });
            }
        ");
        let lib_ast = engine.compile(&lib_code).unwrap();

        Self {
            engine,
            scope,
            output,
            newly_added_bodies,
            existing_bodies,
            commands,
            names: BTreeMap::new(),
            last_code: rhai::AST::default(),
            lib_ast,
            drawings: DrawFn::Finished,
        }
    }
}

impl RhaiRes {
    pub fn run_code(&mut self, code: &str) {
        match self.engine.eval_with_scope::<()>(&mut self.scope, code) {
            Ok(_) => {}
            Err(e) => self
                .output
                .write()
                .unwrap()
                .push_str(e.to_string().as_str()),
        }
    }

    pub fn run_ast(&mut self, code: &rhai::AST) {
        match self
            .engine
            .eval_ast_with_scope::<rhai::Dynamic>(&mut self.scope, code)
        {
            Ok(_) => {}
            Err(e) => self
                .output
                .write()
                .unwrap()
                .push_str(e.to_string().as_str()),
        }
    }
}

pub fn run_code_sys(
    mut code_editor: ResMut<CodeEditor>,
    mut rhai: ResMut<RhaiRes>,
    mut commands: Commands,
) {
    if code_editor.should_run {
        rhai.output.write().unwrap().clear();
        code_editor.should_run = false;

        let code_lock = code_editor.code.read().unwrap();
        let ast = match rhai.engine.compile_with_scope(&rhai.scope, &*code_lock) {
            Ok(ast) => ast.merge(&rhai.lib_ast),
            Err(e) => {
                *rhai.output.write().unwrap() = e.to_string();
                std::mem::drop(e);
                std::mem::drop(code_lock);
                code_editor.output = Some(rhai.output.clone());
                return;
            }
        };
        std::mem::drop(code_lock);
        rhai.run_ast(&ast);
        rhai.last_code = ast;
        code_editor.output = Some(rhai.output.clone());

        for (key, added_body) in rhai.newly_added_bodies.write().unwrap().drain() {
            let registered = added_body
                .get("registered")
                .and_then(|r| r.clone().try_cast::<bool>())
                .unwrap_or(true);

            let mut builder = commands.spawn();

            builder.insert(KinematicBody::from_rhai(added_body));
            if registered {
                builder.insert(RhaiBody);
            }

            let entity = builder.id();
            rhai.existing_bodies.write().unwrap().insert(key, entity);
            commands.entity(entity).insert(RhaiID(key));
        }
    }
}

pub fn run_rhai_commands_sys(
    mut rhai_res: ResMut<RhaiRes>,
    mut query: Query<&mut KinematicBody, With<RhaiBody>>,
    mut g: ResMut<G>,
    mut physics_toggles: ResMut<PhysicsToggles>,
    mut paused: ResMut<Paused>,
) {
    let body_reader = rhai_res.existing_bodies.read().unwrap();
    let mut rhai_commands = rhai_res.commands.write().unwrap();
    let mut draw_fn = DrawFn::Finished;
    for command in rhai_commands.drain(..) {
        match command {
            RhaiCommand::UpdateBody { id, params } => {
                let body_opt = body_reader
                    .get(&id)
                    .and_then(|entity| query.get_mut(*entity).ok());

                if let Some(mut body) = body_opt {
                    macro_rules! generate_set_add {
                        ($field:ident, $str:expr, $ty:ty) => {
                            let set = params
                                .get(concat!("set_", $str))
                                .map(|dynamic| dynamic.clone().try_cast())
                                .flatten();

                            if let Some(field) = set {
                                body.$field = field;
                            }

                            let add = params
                                .get(concat!("add_", $str))
                                .map(|dynamic| dynamic.clone().try_cast::<$ty>())
                                .flatten();

                            if let Some(field) = add {
                                body.$field += field;
                            }
                        };
                    }

                    generate_set_add!(pos, "pos", Vec2);
                    generate_set_add!(vel, "vel", Vec2);
                    generate_set_add!(accel, "accel", Vec2);
                    generate_set_add!(force, "force", Vec2);
                    generate_set_add!(mass, "mass", f32);
                    generate_set_add!(radius, "radius", f32);
                }
            }
            RhaiCommand::Draw { params } => {
                match params.get("type") {
                    Some(s) if s.clone_cast::<String>().as_str() == "line" => {
                        let start = params
                            .get("start")
                            .and_then(|d| d.clone().try_cast::<Vec2>())
                            .unwrap_or(Vec2::ZERO);
                        let end = params
                            .get("end")
                            .and_then(|d| d.clone().try_cast::<Vec2>())
                            .unwrap_or(Vec2::ZERO);
                        let thickness = params
                            .get("thickness")
                            .and_then(|d| d.clone().try_cast::<f32>())
                            .unwrap_or(1.0);
                        // let color = params
                        //     .get("color")
                        //     .and_then(|d| d.clone().try_cast::<Color>())
                        //     .unwrap_or(WHITE);

                        draw_fn = DrawFn::Draw(Arc::new(move || {
                            draw_line(start.x, start.y, end.x, end.y, thickness, WHITE);
                            draw_fn.clone()
                        }))
                    },
                    Some(_) => todo!(),
                    None => todo!(),
                }
            }
            RhaiCommand::SetG(new_g) => {
                g.0 = new_g;
            }
            RhaiCommand::Export => {
                let text = "asdf";
                let dl_link = format!("data:text/plain;charset=utf-8,{text}");
                println!("{}", dl_link);
            }
            RhaiCommand::SetCollisions(enabled_or_disabled) => {
                physics_toggles.collisions = enabled_or_disabled;
            }
            RhaiCommand::SetIntegration(enabled_or_disabled) => {
                physics_toggles.integration = enabled_or_disabled;
            }
            RhaiCommand::DeleteBody { id: _ } => {
                todo!()
            }
            RhaiCommand::SetPaused(enabled_or_disabled) => {
                paused.0 = enabled_or_disabled;
            }
        }
    }

    std::mem::drop(body_reader);
    std::mem::drop(rhai_commands);
    rhai_res.drawings = draw_fn;
}

pub fn run_script_update_sys(
    mut rhai: ResMut<RhaiRes>,
    mut code_editor: ResMut<CodeEditor>,
    dt: Res<DT>,
    paused: Res<Paused>,
    registered_bodies: Query<(Entity, &KinematicBody, &RhaiID)>,
) {
    if let Some(update_fn) = rhai.scope.get_value::<rhai::FnPtr>("update") {
        {
            let existing_bodies_lock = rhai.existing_bodies.clone();
            let mut existing_bodies = existing_bodies_lock.write().unwrap();
            for (e, _, id) in registered_bodies.iter() {
                existing_bodies.insert(id.0, e);
            }
        }

        let registered_bodies_map = {
            Arc::new(
                registered_bodies
                .iter()
                .map(|(e, b, _)| (e, b.clone()))
                .collect::<BTreeMap<Entity, KinematicBody>>(),
            )
        };

        let existing_bodies = rhai.existing_bodies.clone();
        existing_bodies
            .write()
            .unwrap()
            .retain(|_, e| registered_bodies_map.contains_key(&e));
        let existing_body_map = {
            let body_reader = existing_bodies.read().unwrap();
            body_reader
                .iter()
                .map(|(k, e)| {
                    (
                        *k,
                        registered_bodies_map
                        .get(e)
                        .cloned()
                        .map(rhai::Dynamic::from)
                        .unwrap_or(rhai::Dynamic::UNIT),
                    )
                })
            .collect::<BTreeMap<DefaultKey, rhai::Dynamic>>()
        };

        let existing_body_ids = {
            let body_reader = existing_bodies.read().unwrap();
            body_reader.iter().map(|(k, _)| *k).collect::<Vec<_>>()
        };

        rhai.engine.register_fn("get_body", move |id| {
            let body_reader = existing_bodies.read().unwrap();
            body_reader
                .get(&id)
                .and_then(|entity| {
                    registered_bodies_map
                        .get(entity)
                        .cloned()
                        .map(rhai::Dynamic::from)
                })
                .unwrap_or(rhai::Dynamic::UNIT)
        });

        let dt = Arc::new(dt.0);
        rhai.engine.register_fn("DT", move || *dt.clone());

        let paused = Arc::new(paused.0);
        rhai.engine.register_fn("is_paused", move || *paused.clone());

        let ast = rhai.last_code.merge(&rhai.lib_ast);

        let res: Result<rhai::Dynamic, _> =
            update_fn.call(&rhai.engine, &ast, (existing_body_ids, existing_body_map));
        if let Err(e) = res {
            *rhai.output.write().unwrap() = e.to_string();
            rhai.scope.set_value("update", ());
            code_editor.output = Some(rhai.output.clone());
        }
    }
}
