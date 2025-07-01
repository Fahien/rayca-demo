// Copyright © 2020-2025
// Author: Antonio Caggiano <info@antoniocaggiano.eu>
// SPDX-License-Identifier: MIT

use std::ffi::CString;
#[cfg(not(target_os = "android"))]
use std::rc::Rc;

use rayca_core::*;

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

    let shader_ext = if cfg!(target_os = "android") {
        "spv"
    } else {
        "slang"
    };

    let vert_path = format!("shaders/main.vert.{}", shader_ext);
    let frag_path = format!("shaders/main.frag.{}", shader_ext);

    #[cfg(target_os = "android")]
    let (vert, frag) = create_shaders(&win.android_app, &dev.device, &vert_path, &frag_path);
    #[cfg(not(target_os = "android"))]
    let (vert, frag) = create_shaders(&dev.device, &vert_path, &frag_path);

    let entrypoint = CString::new("main").expect("Failed to create main entrypoint");

    let pipeline = DefaultPipeline::new::<Vertex>(
        &mut dev,
        vert.get_stage(&entrypoint, vk::ShaderStageFlags::VERTEX),
        frag.get_stage(&entrypoint, vk::ShaderStageFlags::FRAGMENT),
        &pass,
        win.size.width,
        win.size.height,
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

#[cfg(target_os = "android")]
fn create_shaders(
    android_app: &AndroidApp,
    device: &Rc<ash::Device>,
    vert_path: &str,
    frag_path: &str,
) -> (ShaderModule, ShaderModule) {
    use std::{ffi::CString, str::FromStr};

    let c_vert_path =
        CString::from_str(vert_path).expect("Failed to create CStr for vertex shader path");
    let c_frag_path =
        CString::from_str(frag_path).expect("Failed to create CStr for fragment shader path");
    let mut vert_asset = android_app
        .asset_manager()
        .open(c_vert_path.as_c_str())
        .expect("Failed to open vertex shader");
    let mut frag_asset = android_app
        .asset_manager()
        .open(c_frag_path.as_c_str())
        .expect("Failed to open vertex shader");

    let vert_data = vert_asset
        .buffer()
        .expect("Failed to read vertex shader data");

    let frag_data = frag_asset
        .buffer()
        .expect("Failed to read fragment shader data");

    (
        ShaderModule::from_data(device, vert_data, vk::ShaderStageFlags::VERTEX),
        ShaderModule::from_data(device, frag_data, vk::ShaderStageFlags::FRAGMENT),
    )
}

#[cfg(not(target_os = "android"))]
fn create_shaders(
    device: &Rc<ash::Device>,
    vert_path: &str,
    frag_path: &str,
) -> (ShaderModule, ShaderModule) {
    let vert_data = SlangProgram::get_entry_point_code(vert_path, "main").unwrap();
    let frag_data = SlangProgram::get_entry_point_code(frag_path, "main").unwrap();

    (
        ShaderModule::from_data(device, &vert_data),
        ShaderModule::from_data(device, &frag_data),
    )
}
