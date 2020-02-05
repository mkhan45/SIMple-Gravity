use imgui::*;
use std::fs;
use std::path::Path;

use crate::ecs::systems::graph_sys::GraphType;
use crate::gui::imgui_wrapper::*;
use crate::Vector;

macro_rules! signal_button {
    ( $label:expr, $signal:expr, $ui:expr, $signals:expr) => {
        if $ui.small_button(im_str!($label)) {
            $signals.push($signal);
        }
    };
}

macro_rules! int_slider {
    ( $ui:expr, $label:expr, $num:expr, $min:expr, $max:expr ) => {
        let mut num_i32 = *$num as i32;
        $ui.drag_int(im_str!($label), &mut num_i32)
            .min($min)
            .speed(0.05 * (*$num as f32).powf(1.0 / 3.0))
            .max($max)
            .build();
        *$num = (num_i32 as usize).min($max).max($min);
    };
}

pub fn make_menu_bar(
    ui: &mut imgui::Ui,
    signals: &mut Vec<UiSignal>,
    render_data: &mut RenderData,
) {
    ui.main_menu_bar(|| {
        ui.menu(im_str!("Create Body"), true, || {
            let create_mass = &mut render_data.create_mass;
            let create_rad = &mut render_data.create_rad;

            let mass_speed = (*create_mass * 0.0015).max(0.01);
            let rad_speed = (*create_rad * 0.0015).max(0.01);

            ui.drag_float(im_str!("Mass"), create_mass)
                .speed(mass_speed)
                .build();
            ui.drag_float(im_str!("Radius"), create_rad)
                .speed(rad_speed)
                .build();

            signal_button!("Toggle Create Body", UiSignal::Create, ui, signals);
        });

        ui.separator();

        ui.menu(im_str!("Universal Variables"), true, || {
            let dt = &mut render_data.dt;
            let num_iterations = &mut render_data.num_iterations;
            let preview_iterations = &mut render_data.preview_iterations;

            ui.drag_float(im_str!("Timestep"), dt).speed(0.01).build();
            int_slider!(ui, "Iterations", num_iterations, 1, 1000);
            int_slider!(ui, "Preview Iterations", preview_iterations, 1, 1000);

            signal_button!("Toggle Graphs", UiSignal::ToggleGraphs, ui, signals);
            signal_button!("Toggle Trails", UiSignal::ToggleTrails, ui, signals);
            signal_button!(
                "Toggle Relative Trails",
                UiSignal::ToggleRelativeTrails,
                ui,
                signals
            );
        });

        ui.separator();

        ui.menu(im_str!("Load"), true, || {
            let dir = Path::new("./saved_systems/");
            match fs::read_dir(dir) {
                Ok(dir_entries) => {
                    dir_entries.for_each(|entry| {
                        if let Ok(entry) = entry {
                            if let Ok(filename) = entry.file_name().into_string() {
                                if &filename.as_str()[filename.len() - 4..] == ".ron" {
                                    let label = unsafe {
                                        ImStr::from_utf8_with_nul_unchecked(filename.as_bytes())
                                    };
                                    if ui.small_button(label) {
                                        render_data.load_filename = ImString::new(filename);
                                        signals.push(UiSignal::LoadState);
                                    }
                                }
                            }
                        }
                    });
                }
                Err(e) => println!("Error reading dir: {}", e),
            }
        });

        ui.separator();

        ui.menu(im_str!("Save"), true, || {
            ui.input_text(im_str!("Filename"), &mut render_data.save_filename)
                .build();
            signal_button!("Save the Universe", UiSignal::SaveState, ui, signals);
        });

        ui.separator();

        signal_button!("Reset", UiSignal::DeleteAll, ui, signals);
        ui.separator();
        signal_button!("Pause", UiSignal::Pause, ui, signals);

        ui.separator();
        ui.separator();

        if ui.small_button(im_str!("Help")) {
            ui.open_popup(im_str!("Help Menu"));
        }
        ui.popup_modal(im_str!("Help Menu")).build(|| {
            ui.bullet_text(im_str!("WASD and arrows to move the camera"));
            ui.bullet_text(im_str!("Space to pause"));
            ui.bullet_text(im_str!("F to follow selected body, D to delete it"));
            ui.bullet_text(im_str!("T to toggle trails, G to make them relative"));
            ui.bullet_text(im_str!("Right click a body to edit it"));
            ui.bullet_text(im_str!(
                "Create new bodies and edit universal variables with the top bar"
            ));
            ui.bullet_text(im_str!(
                "Save and load scenarios from the saved_systems folder"
            ));
            if ui.small_button(im_str!("Close Help Menu")) {
                ui.close_current_popup();
            }
        });
    });
}
pub fn make_sidepanel(
    ui: &mut imgui::Ui,
    resolution: Vector,
    open_bool: &mut bool,
    signals: &mut Vec<UiSignal>,
    render_data: &mut RenderData,
) {
    let mass = &mut render_data.mass;
    let rad = &mut render_data.rad;
    let trail_len = &mut render_data.trail_len;
    assert!(render_data.entity_selected);
    // Window
    let win = imgui::Window::new(im_str!("Menu"))
        .position([0.0, 30.0], imgui::Condition::Always)
        .opened(open_bool)
        .size(
            [resolution.x * 0.35, resolution.y - 30.0],
            imgui::Condition::Appearing,
        )
        .collapsible(false)
        .movable(false)
        .size_constraints(
            [resolution.x * 0.1, resolution.y - 30.0],
            [resolution.x * 0.6, resolution.y - 30.0],
        );
    win.build(ui, || {
        //constructs a small button that sends a UiSignal

        ui.text(im_str!("Edit Object"));
        let mass_speed = (*mass * 0.0015).max(0.01);
        let rad_speed = (*rad * 0.0015).max(0.01);
        ui.drag_float(im_str!("Mass"), mass)
            .speed(mass_speed)
            .build();
        ui.drag_float(im_str!("Radius"), rad)
            .speed(rad_speed)
            .build();

        int_slider!(ui, "Trail Length", trail_len, 0, 10_000);

        signal_button!("Follow Body", UiSignal::ToggleFollowBody, ui, signals);
        signal_button!("Delete Body", UiSignal::Delete, ui, signals);

        ui.spacing();
        ui.separator();
        ui.spacing();

        if ui.collapsing_header(im_str!("Graphs")).build() {
            signal_button!(
                "Graph Speed",
                UiSignal::AddGraph(GraphType::Speed),
                ui,
                signals
            );
            signal_button!(
                "Graph X Velocity",
                UiSignal::AddGraph(GraphType::XVel),
                ui,
                signals
            );
            signal_button!(
                "Graph Y Velocity",
                UiSignal::AddGraph(GraphType::YVel),
                ui,
                signals
            );
            signal_button!(
                "Graph Acceleration",
                UiSignal::AddGraph(GraphType::Accel),
                ui,
                signals
            );
        }
    });
}

pub fn make_graph_ui(
    ui: &mut imgui::Ui,
    resolution: Vector,
    open_bool: &mut bool,
    data: &[f32],
    graph_type: GraphType,
) {
    // Window

    let graph_name = match graph_type {
        GraphType::Speed => im_str!("Speed"),
        GraphType::XVel => im_str!("X Velocity"),
        GraphType::YVel => im_str!("Y Velocity"),
        _ => panic!("{:#?} is not an imgui graph", graph_type),
    };

    imgui::Window::new(im_str!("Graphs"))
        .position([resolution.x * 0.6, 0.0], imgui::Condition::Once)
        .size(
            [resolution.x * 0.4, resolution.y * 0.4],
            imgui::Condition::Appearing,
        )
        .opened(open_bool)
        .build(ui, || {
            ui.plot_lines(graph_name, data)
                .graph_size([resolution.x * 0.3, resolution.y * 0.3])
                .build();
        });
}

pub fn make_default_ui(ui: &mut imgui::Ui) {
    // Window
    imgui::Window::new(im_str!("Hello world"))
        .position([100.0, 100.0], imgui::Condition::Appearing)
        .build(ui, || {
            ui.text(im_str!("Hello world!"));
            ui.separator();

            if ui.small_button(im_str!("small button")) {
                println!("Small button clicked");
            }
        });
}
