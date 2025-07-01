// Copyright © 2020-2025
// Author: Antonio Caggiano <info@antoniocaggiano.eu>
// SPDX-License-Identifier: MIT

use rayca_core::*;

rayca_pipe::pipewriter!(Main, "shaders/main.vert.slang", "shaders/main.frag.slang");

impl RenderPipeline for PipelineMain {
    fn render(&self, frame: &Frame, buffer: &Buffer) {
        let graphics_bind_point = vk::PipelineBindPoint::GRAPHICS;
        unsafe {
            self.device.cmd_bind_pipeline(
                frame.cache.command_buffer,
                graphics_bind_point,
                self.get_pipeline(),
            )
        };

        let first_binding = 0;
        let buffers = [buffer.buffer];
        let offsets = [vk::DeviceSize::default()];

        unsafe {
            self.device.cmd_bind_vertex_buffers(
                frame.cache.command_buffer,
                first_binding,
                &buffers,
                &offsets,
            );
        }

        let vertex_count = buffer.size as u32 / std::mem::size_of::<Vertex>() as u32;
        unsafe {
            self.device
                .cmd_draw(frame.cache.command_buffer, vertex_count, 1, 0, 0);
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
    let line_pipeline = PipelineMain::new::<LineVertex>(
        #[cfg(target_os = "android")]
        &win.android_app,
        &pass,
    );

    let lines = vec![
        // Notice how this line appears at the top of the picture as Vulkan Y axis is pointing downwards
        Line::new(
            LineVertex::new(Point3::new(-0.3, -0.3, 0.0), Color::new(1.0, 1.0, 0.0, 1.0)),
            LineVertex::new(Point3::new(0.3, -0.3, 0.0), Color::new(1.0, 1.0, 0.0, 1.0)),
        ),
        Line::new(
            LineVertex::new(Point3::new(0.3, -0.3, 0.0), Color::new(1.0, 0.5, 0.0, 1.0)),
            LineVertex::new(Point3::new(0.3, 0.3, 0.0), Color::new(1.0, 0.5, 0.0, 1.0)),
        ),
        Line::new(
            LineVertex::new(Point3::new(0.3, 0.3, 0.0), Color::new(1.0, 0.1, 0.0, 1.0)),
            LineVertex::new(Point3::new(-0.3, 0.3, 0.0), Color::new(1.0, 0.1, 0.0, 1.0)),
        ),
        Line::new(
            LineVertex::new(Point3::new(-0.3, 0.3, 0.0), Color::new(1.0, 0.0, 0.3, 1.0)),
            LineVertex::new(Point3::new(-0.3, -0.3, 0.0), Color::new(1.0, 0.0, 0.3, 1.0)),
        ),
    ];
    let mut line_buffer =
        Buffer::new::<LineVertex>(&dev.allocator, vk::BufferUsageFlags::VERTEX_BUFFER);
    line_buffer.upload_arr(&lines);

    let mut buffer = Buffer::new::<Vertex>(&dev.allocator, vk::BufferUsageFlags::VERTEX_BUFFER);
    let vertices = [
        Vertex::builder()
            .position(Point3::new(-0.2, -0.2, 0.0))
            .build(),
        Vertex::builder()
            .position(Point3::new(0.2, -0.2, 0.0))
            .build(),
        Vertex::builder()
            .position(Point3::new(0.0, 0.2, 0.0))
            .build(),
    ];
    buffer.upload_arr(&vertices);

    loop {
        events.update(&mut win);
        if win.exit {
            break;
        }

        let frame = match sfs.next_frame() {
            Ok(frame) => frame,
            // Recreate swapchain
            Err(vk::Result::ERROR_OUT_OF_DATE_KHR) => {
                drop(sfs.swapchain);

                sfs.swapchain =
                    Swapchain::new(&vkr.ctx, &surface, &dev, win.size.width, win.size.height);
                for i in 0..sfs.swapchain.images.len() {
                    let frame = &mut sfs.frames[i];
                    frame.buffer = Framebuffer::new(&mut dev, &sfs.swapchain.images[i], &pass);
                }
                continue;
            }
            Err(result) => panic!("{:?}", result),
        };

        frame.begin(&pass);
        main_pipeline.render(frame, &buffer);
        line_pipeline.render(frame, &line_buffer);
        frame.end();

        match sfs.present(&dev) {
            // Recreate swapchain
            Err(vk::Result::ERROR_OUT_OF_DATE_KHR) => {
                drop(sfs.swapchain);

                sfs.swapchain =
                    Swapchain::new(&vkr.ctx, &surface, &dev, win.size.width, win.size.height);
                for i in 0..sfs.swapchain.images.len() {
                    let frame = &mut sfs.frames[i];
                    frame.buffer = Framebuffer::new(&mut dev, &sfs.swapchain.images[i], &pass);
                }
                continue;
            }
            Err(result) => panic!("{:?}", result),
            _ => (),
        }
    }

    dev.wait();
}
