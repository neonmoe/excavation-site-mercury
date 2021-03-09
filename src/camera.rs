pub struct Camera {
    pub x: i32,
    pub y: i32,
}

impl Camera {
    pub fn new() -> Camera {
        Camera { x: 0, y: 0 }
    }

    pub fn update(&mut self, delta_seconds: f32, target_x: i32, target_y: i32) {
        let move_towards = |value: i32, target: i32, speed: i32| {
            let direction_towards_value = (target - value).signum();
            let new_value = value + speed * direction_towards_value;
            if direction_towards_value != (target - new_value).signum() {
                // If the sign changes, this step would "shoot past", so just return the target.
                target
            } else {
                new_value
            }
        };
        let dx = (target_x - self.x) as f32;
        let dy = (target_y - self.y) as f32;
        let camera_distance = (dx * dx + dy * dy).sqrt();
        let camera_movement_speed = camera_distance.powf(1.25) * delta_seconds;
        let camera_movement_speed_x = dx.abs() / camera_distance * camera_movement_speed;
        let camera_movement_speed_y = dy.abs() / camera_distance * camera_movement_speed;
        self.x = move_towards(self.x, target_x, camera_movement_speed_x.max(1.0) as i32);
        self.y = move_towards(self.y, target_y, camera_movement_speed_y.max(1.0) as i32);
    }
}
