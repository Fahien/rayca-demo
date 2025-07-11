// Copyright © 2020-2025
// Author: Antonio Caggiano <info@antoniocaggiano.eu>
// SPDX-License-Identifier: MIT

use rayca_gui::*;
use rayca_pipeline::*;

#[cfg(not(target_os = "android"))]
pub fn main() {
    let win = Win::builder().build();
    main_loop(win);
}

#[cfg(target_os = "android")]
#[unsafe(no_mangle)]
fn android_main(app: AndroidApp) {
    let win = Win::builder().android_app(app).build();
    main_loop(win);
}

fn main_loop(mut win: Win) {
    let mut timer = Timer::new();

    let mut vkr = Vkr::new(&mut win);

    let main_pipeline = PipelineMain::new::<Vertex>(
        #[cfg(target_os = "android")]
        &win.android_app,
        &vkr.pass,
    );
    let line_pipeline = PipelineLine::new::<LineVertex>(
        #[cfg(target_os = "android")]
        &win.android_app,
        &vkr.pass,
    );

    let mut pipelines = Vec::<Box<dyn RenderPipeline>>::new();
    pipelines.push(Box::new(main_pipeline));
    pipelines.push(Box::new(line_pipeline));

    let mut gui = Gui::new(
        #[cfg(target_os = "android")]
        &win.android_app,
        vkr.frames.frames.len(),
        &vkr.dev.allocator,
        &vkr.pass,
    );

    // Get model path from CLI
    let model_path = std::env::args()
        .nth(1)
        .unwrap_or("models/box-textured/BoxTextured.gltf".to_string());

    let gltf_model =
        Model::load_gltf_path(model_path, &vkr.assets).expect("Failed to open gltf model");
    let mut model = RenderModel::new_with_gltf(&vkr.dev, &vkr.assets, gltf_model);

    let camera = model.push_camera(Camera::infinite_perspective(1.0, 3.14 / 4.0, 0.1));
    let camera_node = Node::builder()
        .camera(camera)
        .trs(Trs::builder().translation(Vec3::new(0.0, 0.0, 4.0)).build())
        .build();
    let camera_node_handle = model.push_node(camera_node);
    model.push_to_scene(camera_node_handle);

    let mut current_pipeline = 0;

    loop {
        vkr.update(&mut win);
        if win.exit {
            break;
        }

        let delta = timer.get_delta().as_secs_f32();

        // Move camera
        let camera_node = model.get_node_mut(camera_node_handle).unwrap();
        let mut camera_z_move = 0.0;
        if win.input.w.is_down() {
            camera_z_move -= 1.0 * delta;
        }
        if win.input.s.is_down() {
            camera_z_move += 1.0 * delta;
        }
        camera_node
            .trs
            .translate(Vec3::new(0.0, 0.0, camera_z_move));

        {
            // Update camera
            let camera = model.get_camera_mut(camera).unwrap();
            *camera = Camera::orthographic(4.0, 4.0, 0.1, 10.0);
        }

        let frame = vkr.next_frame(&win).unwrap();
        let Some(mut frame) = frame else {
            continue;
        };

        frame.begin(&model);

        let gui_ctx = gui.begin(delta, &win.input, frame.get_size());

        egui::Window::new("Switch")
            .auto_sized()
            .collapsible(false)
            .fixed_pos(egui::pos2(32.0, 32.0))
            .show(gui_ctx, |ui| {
                ui.radio_value(&mut current_pipeline, 0, "present");
                ui.radio_value(&mut current_pipeline, 1, "normal");
                ui.radio_value(&mut current_pipeline, 2, "depth");
            });

        gui.end(&mut frame);

        frame.begin_render(&vkr.pass);
        frame.draw(&model, &pipelines);

        match current_pipeline {
            0 => frame.end_scene(&vkr.present_pipeline),
            1 => frame.end_scene(&vkr.normal_pipeline),
            2 => frame.end_scene(&vkr.depth_pipeline),
            _ => unreachable!(),
        };

        gui.draw(&mut frame);
        vkr.present(&win, frame).unwrap();
    }

    // Make sure device is idle before releasing Vulkan resources
    vkr.dev.wait();
}
