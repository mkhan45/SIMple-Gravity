use crate::scripting::RhaiRes;
use egui_macroquad::egui::{self, Context};
use egui_macroquad::macroquad::prelude::*;
use egui_plot::{Line, Plot, PlotPoints, Legend};
use bevy_ecs::prelude::*;

use std::collections::VecDeque;

#[derive(Clone, Debug)]
pub struct Graph {
    pub points: VecDeque<f32>,
    pub max_points: usize,
    pub label: String,
    pub color: [u8; 3],
}

impl Graph {
    pub fn new(name: &str, max_points: usize, r: u8, g: u8, b: u8) -> Graph {
        Graph {
            points: VecDeque::new(),
            max_points,
            label: name.to_string(),
            color: [r, g, b],
        }
    }
}

pub fn draw_graphs_sys(
    rhai: Res<RhaiRes>,
    egui_ctx: Res<Context>,
) {
    let graphs_ref = rhai.graphs.clone();
    let graphs = graphs_ref.read().unwrap();
    if graphs.is_empty() {
        return;
    }

    egui::Window::new("Graphs")
        .default_pos(egui::Pos2::new(screen_width() * 0.75, 0.0))
        .default_width(screen_width() * 0.2)
        .default_height(screen_width() * 0.2)
        .show(&egui_ctx, |ui| {
            let legend = Legend::default();

            Plot::new("Graphs")
                .legend(legend)
                .show_x(false)
                .show(ui, |plot_ui| {
                    for graph in graphs.values() {
                        let points = graph.points.iter().enumerate().map(|(i, p)| [i as f64, *p as f64]);
                        let plot_points = PlotPoints::new(points.collect());

                        let ecolor = egui::epaint::ecolor::Color32::from_rgb(graph.color[0], graph.color[1], graph.color[2]);
                        let line = Line::new(plot_points).color(ecolor).name(&graph.label);
                        plot_ui.line(line);
                    }
                });
        });
}
