use rhai::{Engine, Scope};
use bevy_ecs::prelude::*;

use crate::ui::code_editor::CodeEditor;

use std::sync::{Arc, RwLock};

pub struct RhaiRes {
    pub engine: Engine,
    pub scope: Scope<'static>,
    pub output: Arc<RwLock<String>>,
}

impl Default for RhaiRes {
    fn default() -> Self {
        let mut scope = Scope::new();
        scope.push("_console_output", String::new());

        let mut engine = Engine::new();
        let output = Arc::new(RwLock::new(String::new()));

        let logger = output.clone();
        engine.on_print(move |s| logger.write().unwrap().push_str(s));

        Self { engine, scope, output }
    }
}

impl RhaiRes {
    pub fn run_code(&mut self, code: &str) {
        match self.engine.eval_with_scope::<()>(&mut self.scope, code) {
            Ok(_) => {}
            Err(e) => println!("{}", e),
        }
    }
}

pub fn run_code_sys(mut code_editor: ResMut<CodeEditor>, mut rhai: ResMut<RhaiRes>) {
    if code_editor.should_run {
        rhai.output.write().unwrap().clear();

        {
            let code = code_editor.code.lock().unwrap();
            rhai.run_code(&code);
        }

        code_editor.output = Some(rhai.output.clone());
        code_editor.should_run = false;
    }
}
