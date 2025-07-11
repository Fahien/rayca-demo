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
    let (width, height) = (win.size.width, win.size.height);

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

    let mut model = RenderModel::new(&vkr.dev);

    let camera = model.push_camera(Camera::orthographic(
        width as f32 / 480.0,
        height as f32 / 480.0,
        0.1,
        1.0,
    ));
    let camera_node = Node::builder()
        .camera(camera)
        .trs(Trs::builder().translation(Vec3::new(0.0, 0.0, 0.0)).build())
        .build();
    let camera_node_handle = model.push_node(camera_node);
    model.push_to_scene(camera_node_handle);

    let image = Image::builder().uri("images/test.png").build();
    let image_handle = model.push_image(image, &vkr.assets);
    let sampler_handle = model.push_sampler(Sampler::default());
    let texture_handle = model.push_texture(Texture::new(image_handle, sampler_handle));

    let lines_material = model.push_material(Material::builder().shader(1).build());

    let lines_primitive = {
        // Notice how the first line appears at the top of the picture as Vulkan Y axis is pointing downwards
        let vertices = vec![
            Vertex::builder()
                .position(Point3::new(-0.5, -0.5, 0.0))
                .color(Color::new(1.0, 1.0, 0.0, 1.0))
                .build(),
            Vertex::builder()
                .position(Point3::new(0.5, -0.5, 0.0))
                .color(Color::new(1.0, 1.0, 0.0, 1.0))
                .build(),
            Vertex::builder()
                .position(Point3::new(0.5, 0.5, 0.0))
                .color(Color::new(1.0, 0.5, 0.0, 1.0))
                .build(),
            Vertex::builder()
                .position(Point3::new(-0.5, 0.5, 0.0))
                .color(Color::new(1.0, 0.1, 0.0, 1.0))
                .build(),
            Vertex::builder()
                .position(Point3::new(-0.5, -0.5, 0.0))
                .color(Color::new(1.0, 0.0, 0.3, 1.0))
                .build(),
        ];
        Primitive::builder()
            .vertices(vertices)
            .mode(PrimitiveMode::Lines)
            .material(lines_material)
            .build()
    };

    let lines_primitive_handle = model.push_primitive(lines_primitive);
    let lines_mesh = model.push_mesh(Mesh::builder().primitive(lines_primitive_handle).build());
    let lines = model.push_node(
        Node::builder()
            .trs(Trs::builder().translation(Vec3::new(0.5, 0.5, 0.3)).build())
            .mesh(lines_mesh)
            .build(),
    );
    model.push_to_scene(lines);

    let rect_material = model.push_material(
        Material::builder()
            .texture(texture_handle.id.into())
            .build(),
    );

    let rect_primitive = {
        let vertices = vec![
            Vertex::builder()
                .position(Point3::new(-0.2, -0.2, 0.0))
                .uv(Vec2::new(0.0, 0.0))
                .build(),
            Vertex::builder()
                .position(Point3::new(0.2, -0.2, 0.0))
                .uv(Vec2::new(1.0, 0.0))
                .build(),
            Vertex::builder()
                .position(Point3::new(-0.2, 0.2, 0.0))
                .uv(Vec2::new(0.0, 1.0))
                .build(),
            Vertex::builder()
                .position(Point3::new(0.2, 0.2, 0.0))
                .uv(Vec2::new(1.0, 1.0))
                .build(),
        ];
        let indices: Vec<u8> = vec![0, 1, 2, 1, 3, 2];

        Primitive::builder()
            .vertices(vertices)
            .indices(
                PrimitiveIndices::builder()
                    .indices(indices)
                    .index_type(ComponentType::U8)
                    .build(),
            )
            .material(rect_material)
            .build()
    };

    let rect_primitive_handle = model.push_primitive(rect_primitive);
    let rect_mesh = model.push_mesh(Mesh::builder().primitive(rect_primitive_handle).build());

    let rect = model.push_node(
        Node::builder()
            .trs(Trs::builder().translation(Vec3::new(0.0, 0.0, 0.2)).build())
            .mesh(rect_mesh)
            .build(),
    );
    model.push_to_scene(rect);

    let cyan_material_handle = model.push_material(
        Material::builder()
            .color(Color::CYAN)
            .texture(texture_handle.id.into())
            .shader(0)
            .build(),
    );
    let cube_primitive_handle = model.push_primitive(
        Primitive::builder()
            .cube()
            .material(cyan_material_handle)
            .build(),
    );
    let cube_mesh = model.push_mesh(Mesh::builder().primitive(cube_primitive_handle).build());

    let cube = model.push_node(
        Node::builder()
            .trs(
                Trs::builder()
                    .translation(Vec3::new(-0.5, -0.5, 0.0))
                    .build(),
            )
            .mesh(cube_mesh)
            .build(),
    );
    model.push_to_scene(cube);

    let mut current_pipeline = 0;

    loop {
        vkr.update(&mut win);
        if win.exit {
            break;
        }

        let delta = timer.get_delta().as_secs_f32();

        let rot = Quat::axis_angle(Vec3::new(0.0, 0.0, 1.0), delta / 2.0);
        model.get_node_mut(rect).unwrap().trs.rotate(rot);

        let rot = Quat::axis_angle(Vec3::new(0.0, 0.0, 1.0), -delta / 2.0);
        model.get_node_mut(lines).unwrap().trs.rotate(rot);

        {
            // Update camera
            let camera = model.get_camera_mut(camera).unwrap();
            *camera = Camera::orthographic(
                win.size.width as f32 / 480.0,
                win.size.height as f32 / 480.0,
                0.1,
                1.0,
            );
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
            });

        gui.end(&mut frame);

        frame.begin_render(&vkr.pass);
        frame.draw(&model, &pipelines);

        if current_pipeline == 0 {
            frame.end_scene(&vkr.present_pipeline);
        } else {
            frame.end_scene(&vkr.normal_pipeline);
        };

        gui.draw(&mut frame);
        vkr.present(&win, frame).unwrap();
    }

    // Make sure device is idle before releasing Vulkan resources
    vkr.dev.wait();
}
