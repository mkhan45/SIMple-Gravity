use bevy_ecs::prelude::*;
use egui_macroquad::macroquad::prelude::*;
use rhai::{Engine, Scope};

use crate::{physics::KinematicBody, ui::code_editor::CodeEditor};

use slotmap::{DefaultKey, SlotMap};

use std::{
    collections::BTreeMap,
    sync::{Arc, RwLock},
};

pub struct RhaiRes {
    pub engine: Engine,
    pub scope: Scope<'static>,
    pub output: Arc<RwLock<String>>,
    pub newly_added_bodies: Arc<RwLock<SlotMap<DefaultKey, KinematicBody>>>,
    pub existing_bodies: Arc<RwLock<BTreeMap<DefaultKey, Entity>>>,
}

impl Default for RhaiRes {
    fn default() -> Self {
        let mut scope = Scope::new();
        scope.push("_console_output", String::new());
        // scope.push("_new_bodies", SlotMap::<DefaultKey, KinematicBody>::new());

        let mut engine = Engine::new();
        let output = Arc::new(RwLock::new(String::new()));

        let logger = output.clone();
        engine.on_print(move |s| logger.write().unwrap().push_str(s));

        engine.register_type::<KinematicBody>();
        engine.register_type::<Vec2>();
        engine.register_type::<Entity>();
        engine.register_type::<DefaultKey>();
        engine.register_type::<SlotMap<DefaultKey, KinematicBody>>();

        engine.register_fn("insert", SlotMap::<DefaultKey, KinematicBody>::insert);
        engine.register_fn("get", SlotMap::<DefaultKey, KinematicBody>::get);

        engine.register_fn("vec", Vec2::new);
        engine.register_fn("new_body", KinematicBody::from_rhai);

        let newly_added_bodies = Arc::new(RwLock::new(SlotMap::<DefaultKey, KinematicBody>::new()));
        let existing_bodies = Arc::new(RwLock::new(BTreeMap::<DefaultKey, Entity>::new()));

        let new_bodies_ref = newly_added_bodies.clone();
        engine.register_fn("add_body", move |body| {
            new_bodies_ref
                .write()
                .unwrap()
                .insert(KinematicBody::from_rhai(body))
        });

        Self {
            engine,
            scope,
            output,
            newly_added_bodies,
            existing_bodies,
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
) {
    if code_editor.should_run {
        rhai.output.write().unwrap().clear();

        {
            let code = code_editor.code.lock().unwrap();
            rhai.run_code(&code);
        }

        for (key, added_body) in rhai.newly_added_bodies.write().unwrap().drain() {
            let entity = commands.spawn()
                .insert(added_body)
                .id();
            rhai.existing_bodies.write().unwrap().insert(key, entity);
        }

        code_editor.output = Some(rhai.output.clone());
        code_editor.should_run = false;
    }
}
