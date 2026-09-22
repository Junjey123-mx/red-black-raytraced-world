use raylib::prelude::*;

use crate::config;

pub fn run() {
    let (mut rl, thread) = raylib::init()
        .size(config::WINDOW_WIDTH, config::WINDOW_HEIGHT)
        .title(config::WINDOW_TITLE)
        .build();

    while !rl.window_should_close() {
        let mut d = rl.begin_drawing(&thread);
        d.clear_background(Color::BLACK);
    }
}
