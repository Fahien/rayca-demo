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

    let mut model = RenderModel::default();

    let camera = model.gltf.cameras.push(Camera::orthographic(
        width as f32 / 480.0,
        height as f32 / 480.0,
        0.1,
        1.0,
    ));
    let camera_node = Node::builder()
        .camera(camera)
        .trs(Trs::builder().translation(Vec3::new(0.0, 0.0, 0.0)).build())
        .build();
    let camera_node_handle = model.gltf.nodes.push(camera_node);
    model.gltf.scene.push(camera_node_handle);

    let asset = Asset::load(
        #[cfg(target_os = "android")]
        &win.android_app,
        "images/test.png",
    );
    let mut png = Png::new(asset);
    let image = RenderImage::load(&vkr.dev, &mut png);
    let view = ImageView::new(&vkr.dev.device.device, &image);
    let sampler = RenderSampler::new(&vkr.dev.device.device);
    let texture = RenderTexture::new(&view, &sampler);

    model.images.push(image);
    model.views.push(view);
    model.samplers.push(sampler);
    let texture_handle = model.textures.push(texture);

    let line_primitives = {
        // Notice how the first line appears at the top of the picture as Vulkan Y axis is pointing downwards
        let lines_vertices = vec![
            LineVertex::new(Point3::new(-0.5, -0.5, 0.0), Color::new(1.0, 1.0, 0.0, 1.0)),
            LineVertex::new(Point3::new(0.5, -0.5, 0.0), Color::new(1.0, 1.0, 0.0, 1.0)),
            LineVertex::new(Point3::new(0.5, 0.5, 0.0), Color::new(1.0, 0.5, 0.0, 1.0)),
            LineVertex::new(Point3::new(-0.5, 0.5, 0.0), Color::new(1.0, 0.1, 0.0, 1.0)),
            LineVertex::new(Point3::new(-0.5, -0.5, 0.0), Color::new(1.0, 0.0, 0.3, 1.0)),
        ];
        RenderPrimitive::new(&vkr.dev.allocator, &lines_vertices)
    };
    model.primitives.push(line_primitives);

    let lines_material = model
        .gltf
        .materials
        .push(Material::builder().shader(1).build());

    let lines_primitive_handle = model
        .gltf
        .primitives
        .push(Primitive::builder().material(lines_material).build());
    let lines_mesh = model
        .gltf
        .meshes
        .push(Mesh::builder().primitive(lines_primitive_handle).build());
    let lines = model.gltf.nodes.push(
        Node::builder()
            .trs(Trs::builder().translation(Vec3::new(0.5, 0.5, 0.3)).build())
            .mesh(lines_mesh)
            .build(),
    );
    model.gltf.scene.push(lines);

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
        let mut primitive = RenderPrimitive::new(&vkr.dev.allocator, &vertices);
        let indices = vec![0, 1, 2, 1, 3, 2];
        primitive.set_indices(&indices);
        primitive
    };

    let rect_material = model.gltf.materials.push(
        Material::builder()
            .texture(texture_handle.id.into())
            .build(),
    );
    let rect_primitive_handle = model
        .gltf
        .primitives
        .push(Primitive::builder().material(rect_material).build());
    let rect_mesh = model
        .gltf
        .meshes
        .push(Mesh::builder().primitive(rect_primitive_handle).build());
    model.primitives.push(rect_primitive);

    let rect = model.gltf.nodes.push(
        Node::builder()
            .trs(Trs::builder().translation(Vec3::new(0.0, 0.0, 0.2)).build())
            .mesh(rect_mesh)
            .build(),
    );
    model.gltf.scene.push(rect);

    let cyan_material_handle = model
        .gltf
        .materials
        .push(Material::builder().color(Color::CYAN).shader(0).build());
    let cube_primitive = RenderPrimitive::cube(&vkr.dev.allocator);
    let cube_primitive_handle = model
        .gltf
        .primitives
        .push(Primitive::builder().material(cyan_material_handle).build());
    let cube_mesh = model
        .gltf
        .meshes
        .push(Mesh::builder().primitive(cube_primitive_handle).build());
    model.primitives.push(cube_primitive);

    let cube = model.gltf.nodes.push(
        Node::builder()
            .trs(
                Trs::builder()
                    .translation(Vec3::new(-0.5, -0.5, 0.0))
                    .build(),
            )
            .mesh(cube_mesh)
            .build(),
    );
    model.gltf.scene.push(cube);

    loop {
        vkr.update(&mut win);
        if win.exit {
            break;
        }

        let delta = timer.get_delta().as_secs_f32();

        let rot = Quat::axis_angle(Vec3::new(0.0, 0.0, 1.0), delta / 2.0);
        model.gltf.nodes.get_mut(rect).unwrap().trs.rotate(rot);

        let rot = Quat::axis_angle(Vec3::new(0.0, 0.0, 1.0), -delta / 2.0);
        model.gltf.nodes.get_mut(lines).unwrap().trs.rotate(rot);

        {
            // Update camera
            let camera = model.gltf.cameras.get_mut(camera).unwrap();
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
        gui.update(delta, &win.input, &mut frame);
        frame.begin_render(&vkr.pass);
        frame.draw(&model, &pipelines);
        gui.draw(&mut frame);
        frame.end();

        vkr.present(&win, frame).unwrap();
    }

    // Make sure device is idle before releasing Vulkan resources
    vkr.dev.wait();
}
