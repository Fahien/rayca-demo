// Copyright © 2020-2025
// Author: Antonio Caggiano <info@antoniocaggiano.eu>
// SPDX-License-Identifier: MIT

use crate::*;

#[derive(Default)]
pub struct Panel {
    pub current_pipeline: usize,
}

impl Panel {
    fn show_node(ui: &mut egui::Ui, model: &Model, node: Handle<Node>, indent: usize) {
        let node = model.nodes.get(node).unwrap();
        let indent_str = " ".repeat(indent * 2);
        if let Some(name) = node.name.as_ref() {
            ui.label(format!("{indent_str}Node: {}", name));
        } else {
            ui.label(format!("{indent_str}Node: Unknown"));
        }
        ui.label(format!("{indent_str}Transform: {:?}", node.trs));
        for child in node.children.iter() {
            Self::show_node(ui, model, *child, indent + 1);
        }
    }

    pub fn show(&mut self, delta: f32, win: &Win, frame: &mut Frame, gui: &mut Gui) {
        let gui_ctx = gui.begin(delta, &win.input, win.size);
        let frame_size = frame.get_size();

        egui::Window::new("Frame")
            .collapsible(false)
            .auto_sized()
            .show(gui_ctx, |ui| {
                ui.label(format!(
                    "Win size: {}x{}\nFrame size: {}x{}",
                    win.size.width, win.size.height, frame_size.width, frame_size.height
                ));
            });

        egui::Window::new("Input")
            .collapsible(false)
            .auto_sized()
            .show(gui_ctx, |ui| {
                ui.label(format!(
                    "Left Axis: {:.2}x{:.2}",
                    win.input.android.left_axis.x, win.input.android.left_axis.y
                ));
            });

        gui.end(frame);
    }
}
