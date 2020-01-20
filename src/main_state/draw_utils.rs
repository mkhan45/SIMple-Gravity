use ggez::{
    graphics,
    graphics::{Color, DrawMode, MeshBuilder},
    Context,
};

use specs::prelude::*;

use crate::ecs::components::{
    Draw, Mass, Position, Preview, Radius, SpeedGraph, Trail, XVelGraph, YVelGraph,
};
use crate::ecs::resources::{MainIterations, PreviewIterations, Resolution, StartPoint, DT};
use crate::ecs::systems::graph_sys::GraphType;
use crate::main_state::state::{scale_pos, MainState};

static TRAIL_COLOR: graphics::Color = graphics::Color::new(0.25, 0.45, 1.0, 1.0);

impl<'a, 'b> MainState<'a, 'b> {
    pub fn update_gui_data(&mut self) {
        if let Some(e) = self.selected_entity {
            let masses = self.world.read_storage::<Mass>();
            let radii = self.world.read_storage::<Radius>();
            let trails = self.world.read_storage::<Trail>();
            self.imgui_wrapper.render_data.mass = masses.get(e).unwrap().0;
            self.imgui_wrapper.render_data.rad = radii.get(e).unwrap().0;
            self.imgui_wrapper.render_data.trail_len = trails.get(e).unwrap().max_len;
        }
    }

    pub fn draw_trails(&self, builder: &mut MeshBuilder) {
        let trails = self.world.read_storage::<Trail>();
        let radii = self.world.read_storage::<Radius>();

        (&trails, &radii).join().for_each(|(trail, radius)| {
            let slices = trail.points.as_slices();
            if slices.0.len() >= 2 {
                if let Err(e) = builder.line(slices.0, 0.25 * radius.0, TRAIL_COLOR) {
                    dbg!(e);
                };
            }
            if slices.1.len() >= 2 {
                if let Err(e) = builder.line(slices.1, 0.25 * radius.0, TRAIL_COLOR) {
                    dbg!(e);
                };
            }
        });
    }

    pub fn draw_bodies(&self, builder: &mut MeshBuilder) {
        // this is kind of inelegant but previews don't have the Draw component and color is
        // hardcoded
        // TODO?
        let draws = self.world.read_storage::<Draw>();
        let positions = self.world.read_storage::<Position>();
        let radii = self.world.read_storage::<Radius>();
        let previews = self.world.read_storage::<Preview>();

        (&draws, &positions, &radii)
            .join()
            .for_each(|(color, pos, rad)| {
                let point: ggez::mint::Point2<f32> = (*pos).into();
                builder.circle(DrawMode::fill(), point, rad.0, 0.01, color.0);
            });

        (&previews, &positions, &radii)
            .join()
            .for_each(|(_, pos, rad)| {
                let point: ggez::mint::Point2<f32> = (*pos).into();
                let color = Color::new(0.1, 1.0, 0.2, 1.0);
                builder.circle(DrawMode::fill(), point, rad.0, 0.05, color);
            });
    }

    pub fn draw_preview(&self, builder: &mut MeshBuilder, ctx: &mut Context) {
        let start_point = self.world.fetch::<StartPoint>().0;
        let resolution = self.world.fetch::<Resolution>().0;

        let p = if let Some(start_pos) = start_point {
            start_pos
        } else {
            let mouse_pos = ggez::input::mouse::position(ctx);
            let coords = ggez::graphics::screen_coordinates(ctx);
            scale_pos(mouse_pos, coords, resolution)
        };

        if self.creating {
            builder.circle(
                DrawMode::fill(),
                p,
                self.rad,
                0.05,
                Color::new(1.0, 1.0, 1.0, 0.5),
            );

            if let Some(p) = start_point {
                let mouse_pos = ggez::input::mouse::position(ctx);
                let coords = ggez::graphics::screen_coordinates(ctx);
                let scaled_pos = scale_pos(mouse_pos, coords, resolution);
                builder
                    .line(&[p, scaled_pos], 0.5, graphics::WHITE)
                    .expect("not enough points in line");
            }
        }
    }

    pub fn update_sim_data(&mut self) {
        self.mass = self.imgui_wrapper.render_data.mass;
        self.rad = self.imgui_wrapper.render_data.rad;
        self.world
            .insert::<DT>(DT(self.imgui_wrapper.render_data.dt));
        self.world.insert::<MainIterations>(MainIterations(
            self.imgui_wrapper.render_data.num_iterations,
        ));
        self.world.insert::<PreviewIterations>(PreviewIterations(
            self.imgui_wrapper.render_data.preview_iterations,
        ));
    }

    pub fn draw_gui(&mut self, ctx: &mut Context) {
        let hidpi_factor = self.hidpi_factor;

        let mut graph_data: Vec<(GraphType, &[f32])> = Vec::new();

        // this can probably be done better
        let speed_graphs = self.world.read_storage::<SpeedGraph>();
        let xvel_graphs = self.world.read_storage::<XVelGraph>();
        let yvel_graphs = self.world.read_storage::<YVelGraph>();

        // adds graph data to gui
        macro_rules! register_graph_data {
            ( $query:ident, $component:ty, $graph_type:expr ) => {
                $query.join().filter(|data| data.display).for_each(|data| {
                    graph_data.push(($graph_type, &data.data[..]));
                });
            };
        }

        register_graph_data!(speed_graphs, SpeedGraph, GraphType::Speed);
        register_graph_data!(xvel_graphs, XVelGraph, GraphType::XVel);
        register_graph_data!(yvel_graphs, YVelGraph, GraphType::YVel);

        if let Some(e) = self.selected_entity {
            if self.world.is_alive(e) {
                self.imgui_wrapper
                    .render(ctx, hidpi_factor, &mut self.items_hovered, graph_data);

                {
                    let mut masses_mut = self.world.write_storage::<Mass>();
                    let mut radii_mut = self.world.write_storage::<Radius>();
                    let mut trails_mut = self.world.write_storage::<Trail>();

                    masses_mut
                        .insert(e, Mass(self.imgui_wrapper.render_data.mass))
                        .unwrap_or(None);
                    radii_mut
                        .insert(e, Radius(self.imgui_wrapper.render_data.rad))
                        .unwrap_or(None);
                    trails_mut.get_mut(e).unwrap().max_len =
                        self.imgui_wrapper.render_data.trail_len;
                }

                self.world.entities().entity(e.id());
            } else {
                self.selected_entity = None;
            }
        } else {
            self.imgui_wrapper
                .render(ctx, hidpi_factor, &mut self.items_hovered, graph_data);
        }
    }
}
