use macroquad::prelude::*;

#[derive(Clone, Copy)]
pub struct CameraHandler {
    camera: Camera2D,
    follow_target: bool,
    target_to_follow: (f32, f32),
}

impl CameraHandler {
    pub fn new() -> CameraHandler {
        CameraHandler {
            camera: Camera2D {
                target: vec2(screen_width() / 2.0, screen_height() / 2.0),
                zoom: vec2(2.0 / screen_width(), -2.0 / screen_height()),
                ..Default::default()
            },
            follow_target: false,
            target_to_follow: (0.0, 0.0),
        }
    }

    pub fn update(&mut self) {
        // Update Pan
        self.update_pan();
        self.update_rotation();
        self.update_zoom();

        if is_key_down(KeyCode::H) {
            self.camera.target.x = screen_width() / 2.0;
            self.camera.target.y = screen_height() / 2.0;
            self.camera.rotation = 0.0;
            self.camera.zoom = vec2(2.0 / screen_width(), -2.0 / screen_height());
        }
    }

    pub fn reset(&mut self) {
        self.camera.target.x = screen_width() / 2.0;
        self.camera.target.y = screen_height() / 2.0;
        self.camera.rotation = 0.0;
        self.camera.zoom = vec2(2.0 / screen_width(), -2.0 / screen_height());
    }

    pub fn reset_follow(&mut self) {
        self.follow_target = false;
        self.target_to_follow = (0.0, 0.0);
    }

    pub fn get_camera(&self) -> &Camera2D {
        return &self.camera;
    }

    pub fn get_camera_mut(&mut self) -> &mut Camera2D {
        return &mut self.camera;
    }

    fn update_pan(&mut self) {
        if !self.follow_target {
            if is_key_down(KeyCode::W) {
                self.camera.target.y -= 5.0;
            }
            if is_key_down(KeyCode::S) {
                self.camera.target.y += 5.0;
            }
            if is_key_down(KeyCode::A) {
                self.camera.target.x -= 5.0;
            }
            if is_key_down(KeyCode::D) {
                self.camera.target.x += 5.0;
            }
        } else {
            self.camera.target.x += (self.target_to_follow.0 - self.camera.target.x) * 0.1;
            self.camera.target.y += (self.target_to_follow.1 - self.camera.target.y) * 0.1;
        }
    }

    fn update_rotation(&mut self) {
        if is_key_down(KeyCode::Q) {
            self.camera.rotation -= 0.1;
        }
        if is_key_down(KeyCode::E) {
            self.camera.rotation += 0.1;
        }
    }

    fn update_zoom(&mut self) {
        if is_key_down(KeyCode::Z) {
            self.camera.zoom += vec2(0.01 / screen_width(), -0.01 / screen_height());
        }
        if is_key_down(KeyCode::X) {
            self.camera.zoom -= vec2(0.01 / screen_width(), -0.01 / screen_height());
        }
    }

    pub fn set_target(&mut self, pos: (f32, f32)) {
        self.follow_target = true;
        self.target_to_follow = pos;
    }

    pub fn mouse_position(&self) -> (f32, f32) {
        let mouse_position_screen = mouse_position();
        let mouse_position_world = self.camera.screen_to_world(mouse_position_screen.into());

        return (mouse_position_world.x, mouse_position_world.y);
    }
}
