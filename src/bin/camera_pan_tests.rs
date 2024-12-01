use camera::mouse;
use macroquad::prelude::*;

#[macroquad::main("Mouse Position in World")]
async fn main() {
    let camera = Camera2D {
        zoom: vec2(1.0/screen_width(), -1.0/screen_height()),
        target: vec2(screen_width()*0.5, screen_height()*0.5),
        ..Default::default()
    };

    loop {
        // Get the mouse position in world coordinates
        let mouse_screen_pos = vec2(mouse_position().0, mouse_position().1);
        let mouse_world_pos = camera.screen_to_world(mouse_screen_pos);

        // Start the frame
        clear_background(WHITE);

        set_camera(&camera);

        // Draw a circle at the mouse position in the world
        draw_circle(mouse_world_pos.x, mouse_world_pos.y, 10.0, RED);

        println!("[{}, {}], [{}, {}]", mouse_position().0, mouse_position().1, mouse_world_pos.x, mouse_world_pos.y);

        // Draw additional world content (e.g., a grid or shapes)
        draw_text(
            &format!("Mouse: ({:.2}, {:.2})", mouse_world_pos.x, mouse_world_pos.y),
            mouse_world_pos.x + 20.0,
            mouse_world_pos.y - 20.0,
            20.0,
            BLACK,
        );

        // Reset to default camera for UI
        set_default_camera();

        draw_text("Zoom and Pan with Mouse Aligned", 10.0, 20.0, 30.0, DARKGRAY);

        next_frame().await;
    }
}

fn screen_to_world(camera: &Camera2D, screen_coords: Vec2) -> Vec2 {
    let screen_center = vec2(screen_width() / 2.0, screen_height() / 2.0);

    // Transform screen coordinates to world coordinates
    let world_coords = camera.target
        + (screen_coords - screen_center) * vec2(1.0 / camera.zoom.x, -1.0 / camera.zoom.y);

    world_coords
}