use bevy_ecs::prelude::*;
use egui_macroquad::macroquad::prelude::*;
use rhai::{Engine, Scope};

use crate::{physics::KinematicBody, ui::code_editor::CodeEditor};

use slotmap::{DefaultKey, SlotMap};

use std::{
    collections::BTreeMap,
    sync::{Arc, RwLock},
};

pub mod samples;
mod util;

const LIB_CODE: &str = "
    fn add_force(force) { this.update_body(#{add_force: force}) }
    fn add_force(b, force) { update_body(b, #{add_force: force}) }
    fn set_pos(pos) { this.update_body(#{set_pos: pos}) }
    fn set_pos(b, pos) { update_body(b, #{set_pos: pos}) }
    fn get_pos(b) { b.get_body().pos }
    fn get_pos() { this.get_body().pos }
    fn get_vel(b) { b.get_body().vel }
    fn get_vel() { this.get_body().vel }
    fn get_accel(b) { b.get_body().accel }
    fn get_accel() { this.get_body().accel }
    fn get_force(b) { b.get_body().force }
    fn get_force() { this.get_body().force }
    fn get_mass(b) { b.get_body().mass }
    fn get_mass() { this.get_body().mass }
    fn get_radius(b) { b.get_body().radius }
    fn get_radius() { this.get_body().radius }
";

pub enum RhaiCommand {
    UpdateBody { id: DefaultKey, params: rhai::Map }, // TODO: set timestep, add graph, etc.
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
            .register_get_set("y", |v: &mut Vec2| v.y, |v: &mut Vec2, val: f32| v.y = val);
        engine.register_fn("-", |v: Vec2| -v);

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

        let lib_ast = engine.compile(LIB_CODE).unwrap();

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

        let code_lock = code_editor.code.lock().unwrap();
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
                    let get_vec = |field: &str| {
                        params
                            .get(field)
                            .map(|dynamic| dynamic.clone().try_cast::<Vec2>())
                            .flatten()
                    };
                    let _get_f32 = |field: &str| {
                        params
                            .get(field)
                            .map(|dynamic| dynamic.clone().try_cast::<f32>())
                            .flatten()
                    };

                    if let Some(pos) = get_vec("set_pos") {
                        body.pos = pos;
                    }

                    if let Some(force) = get_vec("add_force") {
                        body.force += force;
                    }
                }
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

        let res: Result<rhai::Dynamic, _> = update_fn.call(&rhai.engine, &ast, ());
        if let Err(e) = res {
            *rhai.output.write().unwrap() = e.to_string();
            rhai.scope.set_value("update", ());
            code_editor.output = Some(rhai.output.clone());
        }
    }
}
