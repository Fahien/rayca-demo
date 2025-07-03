// Copyright © 2020-2025
// Author: Antonio Caggiano <info@antoniocaggiano.eu>
// SPDX-License-Identifier: MIT

use rayca_core::*;

rayca_pipe::pipewriter!(Main, "shaders/main.vert.slang", "shaders/main.frag.slang");
rayca_pipe::pipewriter!(Line, "shaders/line.vert.slang", "shaders/line.frag.slang");

impl RenderPipeline for PipelineLine {
    fn render(&self, frame: &mut Frame, model: &RenderModel, nodes: &[Handle<Node>]) {
        self.bind(&frame.cache);

        for node_handle in nodes.iter().cloned() {
            let model_buffer = frame.cache.uniforms.get(&node_handle).unwrap();

            let model_key = DescriptorKey {
                pipeline_layout: self.get_layout(),
                node: node_handle,
                material: Handle::NONE,
            };
            self.bind_model(
                frame.cache.command_buffer,
                &mut frame.cache.descriptors,
                model_key,
                model_buffer,
            );

            let node = model.gltf.nodes.get(node_handle).unwrap();
            let mesh = model.gltf.meshes.get(node.mesh).unwrap();
            let primitive = model.primitives.get(mesh.primitive.id.into()).unwrap();
            self.draw(&frame.cache, primitive);
        }
    }
}

impl RenderPipeline for PipelineMain {
    fn render(&self, frame: &mut Frame, model: &RenderModel, nodes: &[Handle<Node>]) {
        self.bind(&frame.cache);

        // Supposedly, the material is the same for all nodes
        let node = model.gltf.nodes.get(nodes[0]).unwrap();
        let mesh = model.gltf.meshes.get(node.mesh).unwrap();
        let primitive = model.gltf.primitives.get(mesh.primitive).unwrap();
        let material = model.gltf.materials.get(primitive.material).unwrap();
        let texture = model.textures.get(material.texture.id.into()).unwrap();
        // The problem here is that this is caching descriptor set for index 1
        // with the s key as descriptor set index 1.
        // Need to fix
        let image_key = DescriptorKey {
            pipeline_layout: self.get_layout(),
            node: Handle::NONE,
            material: primitive.material,
        };
        self.bind_texture(
            frame.cache.command_buffer,
            &mut frame.cache.descriptors,
            image_key,
            texture,
        );

        for node_handle in nodes.iter().cloned() {
            let model_buffer = frame.cache.uniforms.get(&node_handle).unwrap();
            let model_key = DescriptorKey {
                pipeline_layout: self.get_layout(),
                node: node_handle,
                material: Handle::NONE,
            };
            self.bind_model(
                frame.cache.command_buffer,
                &mut frame.cache.descriptors,
                model_key,
                model_buffer,
            );

            let node = model.gltf.nodes.get(node_handle).unwrap();
            let mesh = model.gltf.meshes.get(node.mesh).unwrap();
            let primitive = model.gltf.primitives.get(mesh.primitive).unwrap();

            let descriptor_key = DescriptorKey {
                pipeline_layout: self.layout,
                node: Handle::NONE,
                material: primitive.material,
            };
            self.bind_texture(
                frame.cache.command_buffer,
                &mut frame.cache.descriptors,
                descriptor_key,
                &model.textures[0],
            );
            let primitive = model.primitives.get(mesh.primitive.id.into()).unwrap();
            self.draw(&frame.cache, primitive);
        }
    }
}

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

    let mut events = Events::new(&mut win);

    let vkr = Vkr::new(&win);

    loop {
        events.update(&mut win);
        if win.window.is_some() || win.exit {
            break;
        }
    }

    let surface = Surface::new(&win, &vkr.ctx);
    let mut dev = Dev::new(&vkr.ctx, Some(&surface));

    let pass = Pass::new(&dev);

    let (width, height) = (win.size.width, win.size.height);
    let mut sfs = SwapchainFrames::new(&vkr.ctx, &surface, &mut dev, width, height, &pass);

    let main_pipeline = PipelineMain::new::<Vertex>(
        #[cfg(target_os = "android")]
        &win.android_app,
        &pass,
    );
    let line_pipeline = PipelineLine::new::<LineVertex>(
        #[cfg(target_os = "android")]
        &win.android_app,
        &pass,
    );

    let mut pipelines = Vec::<Box<dyn RenderPipeline>>::new();
    pipelines.push(Box::new(main_pipeline));
    pipelines.push(Box::new(line_pipeline));

    let mut model = RenderModel::default();

    let asset = Asset::load(
        #[cfg(target_os = "android")]
        &win.android_app,
        "images/test.png",
    );
    let mut png = Png::new(asset);
    let image = RenderImage::load(&dev, &mut png);
    let view = ImageView::new(&dev.device.device, &image);
    let sampler = RenderSampler::new(&dev.device.device);
    let texture = RenderTexture::new(&view, &sampler);

    model.images.push(image);
    model.views.push(view);
    model.samplers.push(sampler);
    let texture_handle = model.textures.push(texture);

    let line_primitives = {
        // Notice how the first line appears at the top of the picture as Vulkan Y axis is pointing downwards
        let lines_vertices = vec![
            LineVertex::new(Point3::new(-0.3, -0.3, 0.0), Color::new(1.0, 1.0, 0.0, 1.0)),
            LineVertex::new(Point3::new(0.3, -0.3, 0.0), Color::new(1.0, 1.0, 0.0, 1.0)),
            LineVertex::new(Point3::new(0.3, 0.3, 0.0), Color::new(1.0, 0.5, 0.0, 1.0)),
            LineVertex::new(Point3::new(-0.3, 0.3, 0.0), Color::new(1.0, 0.1, 0.0, 1.0)),
            LineVertex::new(Point3::new(-0.3, -0.3, 0.0), Color::new(1.0, 0.0, 0.3, 1.0)),
        ];
        RenderPrimitive::new(&dev.allocator, &lines_vertices)
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
    let lines = model
        .gltf
        .nodes
        .push(Node::builder().mesh(lines_mesh).build());
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
        let mut primitive = RenderPrimitive::new(&dev.allocator, &vertices);
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

    let rect = model
        .gltf
        .nodes
        .push(Node::builder().mesh(rect_mesh).build());
    model.gltf.scene.push(rect);

    loop {
        events.update(&mut win);
        if win.exit {
            break;
        }

        let delta = timer.get_delta().as_secs_f32();

        let rot = Quat::axis_angle(Vec3::new(0.0, 0.0, 1.0), delta / 2.0);
        model.gltf.nodes.get_mut(rect).unwrap().trs.rotate(rot);

        let rot = Quat::axis_angle(Vec3::new(0.0, 0.0, 1.0), -delta / 2.0);
        model.gltf.nodes.get_mut(lines).unwrap().trs.rotate(rot);

        if win.resized {
            dev.wait();
            drop(sfs.swapchain);
            // Current must be reset to avoid LAYOUT_UNDEFINED validation errors
            sfs.current = 0;
            sfs.swapchain =
                Swapchain::new(&vkr.ctx, &surface, &dev, win.size.width, win.size.height);
            for i in 0..sfs.swapchain.images.len() {
                let frame = &mut sfs.frames[i];
                // Only this semaphore must be recreated to avoid validation errors
                // The image drawn one is still in use at the moment
                frame.cache.image_ready = Semaphore::new(&dev.device.device);
                frame.buffer = Framebuffer::new(&dev.device.device, &sfs.swapchain.images[i], &pass);
            }
            win.resized = false;
        }

        let frame = sfs.next_frame();

        if frame.is_err() {
            let result = frame.err().unwrap();
            if result != vk::Result::ERROR_OUT_OF_DATE_KHR {
                panic!("{:?}", result);
            }

            dev.wait();
            drop(sfs.swapchain);
            sfs.swapchain =
                Swapchain::new(&vkr.ctx, &surface, &dev, win.size.width, win.size.height);
            for i in 0..sfs.swapchain.images.len() {
                let frame = &mut sfs.frames[i];
                // Only this semaphore must be recreated to avoid validation errors
                // The image drawn one is still in use at the moment
                frame.cache.image_ready = Semaphore::new(&dev.device.device);
                frame.buffer = Framebuffer::new(&dev.device.device, &sfs.swapchain.images[i], &pass);
            }

            continue;
        };

        let frame = frame.unwrap();
        frame.update(&model);

        frame.begin(&pass, win.size);
        frame.draw(&model, &pipelines);
        frame.end();

        match sfs.present(&dev) {
            // Recreate swapchain
            Err(vk::Result::ERROR_OUT_OF_DATE_KHR) => {
                dev.wait();
                drop(sfs.swapchain);
                sfs.swapchain =
                    Swapchain::new(&vkr.ctx, &surface, &dev, win.size.width, win.size.height);
                for i in 0..sfs.swapchain.images.len() {
                    let frame = &mut sfs.frames[i];
                    // Semaphores must be recreated to avoid validation errors
                    frame.cache.image_ready = Semaphore::new(&dev.device.device);
                    frame.cache.image_drawn = Semaphore::new(&dev.device.device);
                    frame.buffer = Framebuffer::new(&dev.device.device, &sfs.swapchain.images[i], &pass);
                }
                continue;
            }
            Err(result) => panic!("{:?}", result),
            _ => (),
        }
    }

    // Make sure device is idle before releasing Vulkan resources
    dev.wait();
}
