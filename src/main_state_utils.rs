use crate::ecs::components::{AccelGraph, Preview, SpeedGraph, XVelGraph, YVelGraph};
use crate::ecs::resources::{MainIterations, Paused, PreviewIterations};
use crate::ecs::systems::graph_sys::GraphType;
use crate::imgui_wrapper::{UiChoice, UiSignal};
use crate::main_state::MainState;
use crate::saveload::save_world;
use specs::prelude::*;

use std::collections::HashSet;

impl<'a, 'b> MainState<'a, 'b> {
    pub fn run_physics_systems(&mut self) {
        let preview_iterations = self.world.fetch::<PreviewIterations>().0;
        if !self.world.fetch::<Paused>().0 {
            let main_iterations = self.world.fetch::<MainIterations>().0;

            // do_physics(&mut self.world, ctx);
            (0..main_iterations).for_each(|_| {
                self.main_dispatcher.dispatch(&self.world);
            });
            self.world.maintain();
        }
        if let Some(e) = self.selected_entity {
            if !self.world.is_alive(e) {
                self.selected_entity = None;
            }
        }

        (0..preview_iterations).for_each(|_| {
            self.preview_dispatcher.dispatch(&self.world);
        });
    }

    pub fn process_gui_signals(&mut self) {
        self.imgui_wrapper
            .sent_signals
            .clone()
            .iter()
            .for_each(|signal| match signal {
                UiSignal::Create => self.creating = !self.creating,
                UiSignal::Delete => {
                    if let Some(e) = self.selected_entity {
                        self.world
                            .delete_entity(e)
                            .expect("error deleting selected_entity");
                        self.selected_entity = None;
                    }
                }
                UiSignal::AddGraph(graph_type) => {
                    // adds graph component to selected entity
                    macro_rules! add_graphs {
                        ( $ent:expr, $gt:expr, $( [$graph_type:pat, $component:ty] ),* ) => {{
                            match $gt {
                                $(
                                    $graph_type => {
                                        let mut graphs = self.world.write_storage::<$component>();
                                        if graphs.get($ent).is_none() {
                                            graphs
                                                .insert($ent, <$component>::new())
                                                .expect("error adding graph");
                                            } else {
                                                graphs.get_mut($ent).unwrap().display = true;
                                        }
                                    },
                                )*
                            };
                        }}}

                    if let Some(e) = self.selected_entity {
                        add_graphs!(
                            e,
                            graph_type,
                            [GraphType::Speed, SpeedGraph],
                            [GraphType::XVel, XVelGraph],
                            [GraphType::YVel, YVelGraph],
                            [GraphType::Accel, AccelGraph]
                        );
                        if !self.imgui_wrapper.shown_menus.contains(&UiChoice::Graph) {
                            self.imgui_wrapper.shown_menus.insert(UiChoice::Graph);
                        }
                    }
                }
                UiSignal::RemoveGraphs => {
                    macro_rules! undisplay_graphs {
                        ( $( $component:ty ),* ) => {
                            $(
                                let mut graphs = self.world.write_storage::<$component>();
                                (&mut graphs).join().for_each(|graph|{
                                    graph.display = false;
                                });
                            )*
                        };
                    }
                    undisplay_graphs!(SpeedGraph, XVelGraph, YVelGraph);
                }
                UiSignal::SaveState => match save_world(&self.world, "world.ron".to_string()) {
                    Ok(()) => println!("Successfully saved the universe"),
                    Err(e) => println!("Error saving the universe: {}", e),
                },
            });
        self.imgui_wrapper.sent_signals.clear();
    }

    // there's gotta be a better way to do this but its performance doesn't matter
    pub fn delete_preview(&mut self) {
        let mut delset: HashSet<Entity> = HashSet::new();
        {
            let previews = self.world.read_storage::<Preview>();
            let entities = self.world.entities();

            (&entities, &previews).join().for_each(|(entity, _)| {
                delset.insert(entity);
            });
        }

        delset.drain().for_each(|entity| {
            self.world
                .delete_entity(entity)
                .expect("error deleting collided preview");
        });
    }
}
