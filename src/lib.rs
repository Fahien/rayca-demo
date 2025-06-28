// Copyright © 2020-2025
// Author: Antonio Caggiano <info@antoniocaggiano.eu>
// SPDX-License-Identifier: MIT

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

    loop {
        events.update(&mut win);
        if win.exit {
            break;
        }
    }
}
