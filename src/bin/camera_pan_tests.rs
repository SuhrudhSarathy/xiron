use std::f32::MIN_POSITIVE;

use camera::mouse;
use macroquad::prelude::*;

#[macroquad::main("Pan with Buttons")]
async fn main() {
    // Initial camera setup
    let mut camera = Camera2D {
        zoom: vec2(1.0 / screen_width(), -1.0 / screen_height()),
        target: vec2(0.0, 0.0),
        ..Default::default()
    };

    let pan_speed = 15.0; // Speed of panning

    let mut mouse_pressed_last_value: Option<(f32, f32)> = None;

    loop {
        // Input handling for buttons
        let screen_width = screen_width();
        let screen_height = screen_height();
        let button_size = 60.0;

        if is_mouse_button_pressed(MouseButton::Left)
        {
            mouse_pressed_last_value = Some(mouse_position());
        }

        if is_mouse_button_down(MouseButton::Left)
        {

            match mouse_pressed_last_value {
                Some((cx, cy)) => {
                    let (px, py) = mouse_position();
                    let delta_x = px - cx;
                    let delta_y = py - cy;

                    println!("Delta : {}, {}", delta_x, delta_y);
                    
                    camera.target.x += pan_speed * delta_x / screen_width;
                    camera.target.y += pan_speed * delta_y / screen_height;
                }
                None => {}
            }
        }

        if is_mouse_button_released(MouseButton::Left)
        {
            mouse_pressed_last_value = None;
        }


        // Start the frame
        clear_background(WHITE);

        set_camera(&camera);

        // Drawing code
        draw_circle(0.0, 0.0, 50.0, RED); // Example: Draw a circle
        draw_text("Pan with Buttons", -screen_width / 2.0 + 20.0, -screen_height / 2.0 + 40.0, 30.0, BLACK);

        // Reset camera to draw UI
        set_default_camera();

        // Draw buttons
        draw_rectangle(0.0, screen_height / 2.0 - button_size / 2.0, button_size, button_size, GRAY); // Left
        draw_text("<", 20.0, screen_height / 2.0 + 10.0, 30.0, BLACK);

        draw_rectangle(screen_width - button_size, screen_height / 2.0 - button_size / 2.0, button_size, button_size, GRAY); // Right
        draw_text(">", screen_width - 40.0, screen_height / 2.0 + 10.0, 30.0, BLACK);

        draw_rectangle(screen_width / 2.0 - button_size / 2.0, 0.0, button_size, button_size, GRAY); // Up
        draw_text("^", screen_width / 2.0 - 10.0, 20.0, 30.0, BLACK);

        draw_rectangle(screen_width / 2.0 - button_size / 2.0, screen_height - button_size, button_size, button_size, GRAY); // Down
        draw_text("v", screen_width / 2.0 - 10.0, screen_height - 40.0, 30.0, BLACK);

        next_frame().await;
    }
}