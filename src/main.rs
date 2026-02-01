use raylib::prelude::*;

fn main() {
    let (mut rl, thread) = raylib::init()
        .size(800, 450)
        .title("Hello raylib (Rust)")
        .build();

    while !rl.window_should_close() {
        let mut d = rl.begin_drawing(&thread);
        d.clear_background(Color::RAYWHITE);
        d.draw_text("It works!", 20, 20, 30, Color::DARKBLUE);
    }
}
