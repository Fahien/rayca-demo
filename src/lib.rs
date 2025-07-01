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

    let pass = Pass::new(&dev);

    let (width, height) = (win.size.width, win.size.height);
    let mut sfs = SwapchainFrames::new(&vkr.ctx, &surface, &mut dev, width, height, &pass);

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

    loop {
        events.update(&mut win);
        if win.exit {
            break;
        }

        let frame = match sfs.next_frame() {
            Ok(frame) => frame,
            // Recreate swapchain
            Err(vk::Result::ERROR_OUT_OF_DATE_KHR) => {
                drop(sfs);
                sfs = SwapchainFrames::new(
                    &vkr.ctx,
                    &surface,
                    &mut dev,
                    win.size.width,
                    win.size.height,
                    &pass,
                );
                continue;
            }
            Err(result) => panic!("{:?}", result),
        };

        frame.begin(&pass);
        pipeline.render(frame, &buffer);
        frame.end();

        match sfs.present(&dev) {
            // Recreate swapchain
            Err(vk::Result::ERROR_OUT_OF_DATE_KHR) => {
                drop(sfs);
                sfs = SwapchainFrames::new(
                    &vkr.ctx,
                    &surface,
                    &mut dev,
                    win.size.width,
                    win.size.height,
                    &pass,
                );
                continue;
            }
            Err(result) => panic!("{:?}", result),
            _ => (),
        }
    }
}
