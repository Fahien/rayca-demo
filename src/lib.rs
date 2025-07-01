// Copyright © 2020-2025
// Author: Antonio Caggiano <info@antoniocaggiano.eu>
// SPDX-License-Identifier: MIT

use std::ffi::CString;
#[cfg(not(target_os = "android"))]
use std::rc::Rc;

use rayca_core::*;

rayca_pipe::pipewriter!(Main, "shaders/main.vert.slang", "shaders/main.frag.slang");

impl RenderPipeline for PipelineMain {
    fn render(&self, frame: &Frame, buffer: &Buffer) {
        let graphics_bind_point = vk::PipelineBindPoint::GRAPHICS;
        unsafe {
            self.device.cmd_bind_pipeline(
                frame.command_buffer,
                graphics_bind_point,
                self.get_pipeline(),
            )
        };

        let first_binding = 0;
        let buffers = [buffer.buffer];
        let offsets = [vk::DeviceSize::default()];
        unsafe {
            self.device.cmd_bind_vertex_buffers(
                frame.command_buffer,
                first_binding,
                &buffers,
                &offsets,
            );
            self.device.cmd_draw(frame.command_buffer, 3, 1, 0, 0);
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

    let swapchain = Swapchain::new(&vkr.ctx, &surface, &dev, win.size.width, win.size.height);

    let pass = Pass::new(&dev);

    // Frames: collection of per-frame resources (device, swapchain, renderpass, command pool)
    let mut frames = Vec::new();
    for image in &swapchain.images {
        frames.push(Frame::new(&mut dev, image, &pass));
    }

    let pipeline = PipelineMain::new::<Vertex>(
        #[cfg(target_os = "android")]
        &win.android_app,
        &pass,
    );

    let mut buffer = Buffer::new(&vkr.ctx, &mut dev);
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
    buffer.upload(vertices.as_ptr(), buffer.size as usize);

    let mut current_frame = 0;

    loop {
        events.update(&mut win);
        if win.exit {
            break;
        }

        // Wait for this frame to be ready
        let frame = &frames[current_frame];
        frame.wait();

        // Get next image
        let (image_index, _) = unsafe {
            swapchain.ext.acquire_next_image(
                swapchain.swapchain,
                u64::MAX,
                frame.image_ready,
                vk::Fence::null(),
            )
        }
        .expect("Failed to acquire Vulkan next image");

        frame.begin(&pass);
        pipeline.render(frame, &buffer);
        frame.end();
        frame.present(&dev, &swapchain, image_index);

        // Update current frame
        current_frame = (current_frame + 1) % swapchain.images.len();
    }
}
