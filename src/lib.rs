// Copyright © 2020-2025
// Author: Antonio Caggiano <info@antoniocaggiano.eu>
// SPDX-License-Identifier: MIT

mod ui;

use rayca_gui::*;
use rayca_pipeline::*;

#[cfg(not(target_os = "android"))]
pub fn main() {
    let win = Win::builder().size(Size2::new(1920, 1024)).build();
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

    let mut scene = RenderScene::new(&vkr.dev);

    push_model(&mut scene, 0, &vkr);
    push_model(&mut scene, 1, &vkr);

    let mut panel = ui::Panel::default();

    loop {
        win.input.update();
        vkr.update(&mut win);
        if win.exit {
            break;
        }

        // Update camera for window size
        {
            let camera = scene.get_default_camera_mut();
            *camera = Camera::finite_perspective(
                win.size.width as f32 / win.size.height as f32,
                3.14 / 4.0,
                0.1,
                100.0,
            );
        }

        let delta = timer.get_delta().as_secs_f32();

        // Move camera
        move_camera(&mut win, &mut scene, delta);

        let frame = vkr.next_frame(&win).unwrap();
        let Some(mut frame) = frame else {
            continue;
        };

        frame.begin(&scene);

        panel.show(delta, &win, &mut frame, &mut gui);

        frame.begin_render(&vkr.pass);
        frame.draw(&scene, &pipelines);

        match panel.current_pipeline {
            0 => frame.end(&scene, &vkr.present_pipeline),
            1 => frame.end(&scene, &vkr.normal_pipeline),
            2 => frame.end(&scene, &vkr.depth_pipeline),
            _ => unreachable!(),
        };

        gui.draw(&mut frame);
        vkr.present(&win, frame).unwrap();
    }

    // Make sure device is idle before releasing Vulkan resources
    vkr.dev.wait();
}

fn push_model(scene: &mut RenderScene, model_id: usize, vkr: &Vkr) {
    static MODEL_PATHS: [&str; 2] = [
        "models/box-textured/BoxTextured.gltf",
        "models/cesium-milk-truck/CesiumMilkTruck.gltf",
    ];
    // Get model path from CLI
    let model_path = std::env::args()
        .nth(model_id + 1)
        .unwrap_or(MODEL_PATHS[model_id].to_string());

    let gltf_model =
        Model::load_gltf_path(model_path, &vkr.assets).expect("Failed to open gltf model");
    let mut model = RenderModel::new_with_gltf(&vkr.dev, &vkr.assets, gltf_model);
    model
        .get_root_mut()
        .trs
        .translate(Vec3::new(model_id as f32 * 2.0, 0.0, 0.0));

    scene.push_model(model);
}

fn move_camera(win: &mut Win, scene: &mut RenderScene, delta: f32) {
    let speed = 5.0 * delta;

    let camera_node = scene.get_default_camera_node_mut();

    // TODO use right axis for rotation
    if win.input.a.is_down() || win.input.android.l1.is_down() {
        camera_node
            .trs
            .rotate(Quat::axis_angle(Vec3::Y_AXIS, delta));
    }
    if win.input.d.is_down() || win.input.android.r1.is_down() {
        camera_node
            .trs
            .rotate(Quat::axis_angle(Vec3::Y_AXIS, -delta));
    }

    let mut camera_x = win.input.android.left_axis.x;

    if win.input.q.is_down() {
        camera_x -= 1.0;
    }
    if win.input.e.is_down() {
        camera_x += 1.0;
    }

    let mut camera_z = win.input.android.left_axis.y;

    if win.input.w.is_down() {
        camera_z -= 1.0
    }
    if win.input.s.is_down() {
        camera_z += 1.0;
    }

    // When convertin to a matrix, the rotation is applied before the translation,
    // se if we want to move the camera in the direction it is facing,
    // we need to rotate the camera translation.
    let camera_translation = Vec3::new(camera_x, 0.0, camera_z)
        .get_normalized()
        .get_rotated(camera_node.trs.rotation);
    camera_node.trs.translate(camera_translation * speed);
}
