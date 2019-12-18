use crate::{Kinematics, SpeedGraph};
use specs::prelude::*;
use ggez::graphics::{MeshBuilder, Rect, Color, DrawMode};

pub struct SpeedGraphSys;

static GRAPH_RECT: Rect = Rect::new(199.0, 1.0, 100.0, 60.0);

impl<'a> System<'a> for SpeedGraphSys {
    type SystemData = (ReadStorage<'a, Kinematics>, WriteStorage<'a, SpeedGraph>);

    fn run(&mut self, (kinematics, mut graphs) : Self::SystemData) {
        (&kinematics, &mut graphs).join().for_each(|(kine, graph)|{
            graph.0.push(kine.vel.norm());
        });
    }
}

pub fn draw_graphs(graph_builder: &mut MeshBuilder, world: &World) {
    let speed_graphs = world.read_storage::<SpeedGraph>();
    graph_builder.rectangle(DrawMode::stroke(1.0), GRAPH_RECT, ggez::graphics::WHITE);
    (&speed_graphs).join().for_each(|graph|{
        draw_line(graph_builder, graph.0.as_slice(), 0.0, 10.0, ggez::graphics::Color::new(1.0, 0.0, 0.0, 1.0));
    });
}

fn draw_line(
    graph_builder: &mut MeshBuilder,
    data: &[f32],
    offset: f32,
    multiplier: f32,
    color: Color,
) {
    if offset != 0.0 {
        graph_builder
            .line(
                &[[21.0, offset], [21.0 + 15.0, offset]],
                0.05,
                Color::new(0.6, 0.6, 0.6, 1.0),
            )
            .expect("error drawing mid graph line");
    }

    if data.len() >= 2 {
        graph_builder
            .line(
                data.iter()
                    .enumerate()
                    .map(|(i, val)| {
                        let mut y_pos = 15.0 - offset - multiplier * *val as f32;
                        if y_pos <= 0.0 {
                            y_pos = 0.0;
                        } else if y_pos >= 15.0 {
                            y_pos = 15.0;
                        }

                        [21.0 + 15.0 * i as f32 / 1500. as f32, y_pos]
                    })
                    .collect::<Vec<_>>()
                    .as_slice(),
                0.05,
                color,
            )
            .expect("error drawing graph line");
    }
}
