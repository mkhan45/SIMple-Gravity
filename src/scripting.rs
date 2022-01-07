use bevy_ecs::prelude::*;
use egui_macroquad::macroquad::prelude::*;
use rhai::{Engine, Scope};

use crate::{
    physics::{KinematicBody, G},
    ui::code_editor::CodeEditor,
};

use slotmap::{DefaultKey, SlotMap};

use std::{
    collections::BTreeMap,
    sync::{Arc, RwLock},
};

pub mod samples;
mod util;

pub enum RhaiCommand {
    UpdateBody { id: DefaultKey, params: rhai::Map }, // TODO: set timestep, add graph, etc.
    SetG(f32),
    Export,
}

pub struct RhaiBody;
pub struct RhaiRes {
    pub engine: Engine,
    pub scope: Scope<'static>,
    pub output: Arc<RwLock<String>>,
    pub newly_added_bodies: Arc<RwLock<SlotMap<DefaultKey, rhai::Map>>>,
    pub existing_bodies: Arc<RwLock<BTreeMap<DefaultKey, Entity>>>,
    pub commands: Arc<RwLock<Vec<RhaiCommand>>>,
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
            .register_get("length", |v: &mut Vec2| v.length());

        engine.register_fn("-", |v: Vec2| -v);
        engine.register_fn("-", |lhs: Vec2, rhs: Vec2| lhs - rhs);
        engine.register_fn("+", |lhs: Vec2, rhs: Vec2| lhs + rhs);
        engine.register_fn("/", |lhs: Vec2, rhs: f32| lhs / rhs);
        engine.register_fn("/", |lhs: Vec2, rhs: i64| lhs / rhs as f32);
        engine.register_fn("*", |lhs: Vec2, rhs: f32| lhs * rhs);
        engine.register_fn("*", |lhs: Vec2, rhs: i64| lhs * rhs as f32);

        engine.register_type::<Entity>();
        engine.register_type::<DefaultKey>();
        engine.register_type::<SlotMap<DefaultKey, KinematicBody>>();

        engine.register_fn("insert", SlotMap::<DefaultKey, KinematicBody>::insert);
        engine.register_fn("get", SlotMap::<DefaultKey, KinematicBody>::get);

        engine.register_fn("vec", Vec2::new);
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
        engine.register_fn("set_g", move |new_g| {
            let mut commands_writer = command_ref.write().unwrap();
            commands_writer.push(RhaiCommand::SetG(new_g));
        });

        // TODO: Constify via a macro
        let mut lib_code = "".to_string();
        for field in ["pos", "vel", "accel", "force", "mass", "radius"] {
            lib_code.push_str(&format!(
                "
                fn get_{0}(body) {{ get_body(body).{0} }}
                fn get_{0}() {{ get_body(this).{0} }}

                fn add_{0}(body, field) {{ update_body(body, #{{ add_{0}: field }}) }}
                fn add_{0}(field) {{ update_body(this, #{{ add_{0}: field }} ) }}

                fn set_{0}(body, field) {{ update_body(body, #{{ set_{0}: field }}) }}
                fn set_{0}(field) {{ update_body(this, #{{ set_{0}: field }} ) }}
            ",
                field
            ));
        }
        let lib_ast = engine.compile(&lib_code).unwrap();

        Self {
            engine,
            scope,
            output,
            newly_added_bodies,
            existing_bodies,
            commands,
            last_code: rhai::AST::default(),
            lib_ast,
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

pub struct RhaiID(pub usize);

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
                .unwrap_or(false);

            let mut builder = commands.spawn();

            builder.insert(KinematicBody::from_rhai(added_body));
            if registered {
                builder.insert(RhaiBody);
            }

            let entity = builder.id();
            rhai.existing_bodies.write().unwrap().insert(key, entity);
        }
    }
}

pub fn run_rhai_commands_sys(
    rhai_res: Res<RhaiRes>,
    mut query: Query<&mut KinematicBody, With<RhaiBody>>,
    mut g: ResMut<G>,
) {
    let body_reader = rhai_res.existing_bodies.read().unwrap();
    let mut rhai_commands = rhai_res.commands.write().unwrap();
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
            RhaiCommand::SetG(new_g) => {
                g.0 = new_g;
            }
            RhaiCommand::Export => {
                let text = "asdf";
                let dl_link = format!("data:text/plain;charset=utf-8,{text}");
                println!("{}", dl_link);
            }
        }
    }
}

pub fn run_script_update_sys(
    mut rhai: ResMut<RhaiRes>,
    mut code_editor: ResMut<CodeEditor>,
    registered_bodies: Query<(Entity, &KinematicBody), With<RhaiBody>>,
) {
    if let Some(update_fn) = rhai.scope.get_value::<rhai::FnPtr>("update") {
        let registered_bodies_map = {
            Arc::new(
                registered_bodies
                    .iter()
                    .map(|(e, b)| (e, b.clone()))
                    .collect::<BTreeMap<Entity, KinematicBody>>(),
            )
        };

        let existing_bodies = rhai.existing_bodies.clone();
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
                            .map(|b| rhai::Dynamic::from(b))
                            .unwrap_or(rhai::Dynamic::UNIT),
                    )
                })
                .collect::<BTreeMap<DefaultKey, rhai::Dynamic>>()
        };

        rhai.engine.register_fn("get_body", move |id| {
            let body_reader = existing_bodies.read().unwrap();
            body_reader
                .get(&id)
                .and_then(|entity| {
                    registered_bodies_map
                        .get(entity)
                        .cloned()
                        .map(|body| rhai::Dynamic::from(body))
                })
                .unwrap_or(rhai::Dynamic::UNIT)
        });

        let ast = rhai.last_code.merge(&rhai.lib_ast);

        let res: Result<rhai::Dynamic, _> = update_fn.call(&rhai.engine, &ast, (existing_body_map,));
        if let Err(e) = res {
            *rhai.output.write().unwrap() = e.to_string();
            rhai.scope.set_value("update", ());
            code_editor.output = Some(rhai.output.clone());
        }
    }
}
