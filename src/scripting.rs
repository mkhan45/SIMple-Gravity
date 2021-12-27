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

pub enum RhaiCommand {
    AddForce { id: DefaultKey, force: Vec2 },
}

pub struct RhaiBody;
pub struct RhaiRes {
    pub engine: Engine,
    pub scope: Scope<'static>,
    pub output: Arc<RwLock<String>>,
    pub newly_added_bodies: Arc<RwLock<SlotMap<DefaultKey, rhai::Map>>>,
    pub existing_bodies: Arc<RwLock<BTreeMap<DefaultKey, Entity>>>,
    pub commands: Arc<RwLock<Vec<RhaiCommand>>>,
}

impl Default for RhaiRes {
    fn default() -> Self {
        let mut scope = Scope::new();
        scope.push("_console_output", String::new());

        let mut engine = Engine::new();
        let output = Arc::new(RwLock::new(String::new()));

        let logger = output.clone();
        engine.on_print(move |s| logger.write().unwrap().push_str(s));

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

        engine.register_type::<Vec2>();
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

        Self {
            engine,
            scope,
            output,
            newly_added_bodies,
            existing_bodies,
            commands,
        }
    }
}

impl RhaiRes {
    pub fn run_code(&mut self, code: &str) {
        match self.engine.eval_with_scope::<()>(&mut self.scope, code) {
            Ok(_) => {}
            Err(e) => *self.output.write().unwrap() = e.to_string(),
        }
    }
}

pub struct RhaiID(pub usize);

pub fn run_code_sys(
    mut code_editor: ResMut<CodeEditor>,
    mut rhai: ResMut<RhaiRes>,
    mut commands: Commands,
    mut query_set: QuerySet<(
        Query<(Entity, &KinematicBody), With<RhaiBody>>,
        Query<&mut KinematicBody, With<RhaiBody>>,
    )>,
) {
    let registered_bodies_map = {
        let registered_bodies = query_set.q0();
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

    let command_ref = rhai.commands.clone();
    rhai.engine.register_fn("add_force", move |id, force| {
        let mut commands_writer = command_ref.write().unwrap();
        commands_writer.push(RhaiCommand::AddForce { id, force });
    });

    if code_editor.should_run {
        rhai.output.write().unwrap().clear();

        {
            let code = code_editor.code.lock().unwrap();
            rhai.run_code(&code);
        }

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

        code_editor.output = Some(rhai.output.clone());
        code_editor.should_run = false;
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
            RhaiCommand::AddForce { id, force } => {
                let body_opt = body_reader.get(&id).and_then(|entity| query.get_mut(*entity).ok());
                if let Some(mut body) = body_opt {
                    body.force += force;
                }
            }
        }
    }
}

pub fn run_script_update_sys(rhai: ResMut<RhaiRes>, code_editor: Res<CodeEditor>) {
    if let Some(update_fn) = rhai.scope.get_value::<rhai::FnPtr>("update") {
        let code = code_editor.code.lock().unwrap();
        let ast = rhai.engine.compile(&*code).unwrap();
        let () = update_fn.call(&rhai.engine, &ast, ()).unwrap();
    }
}
