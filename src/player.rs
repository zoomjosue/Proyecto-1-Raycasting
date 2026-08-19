use crate::map::{GameMap, CELL_SIZE};
use minifb::{Key, MouseMode, Window};
use nalgebra_glm::Vec2;
use std::f32::consts::TAU;

pub struct Player {
    pub position: Vec2,
    pub angle: f32,
    pub radius: f32,
    last_mouse_x: Option<f32>,
}

impl Player {
    pub fn new(position: Vec2) -> Self {
        Self {
            position,
            angle: 0.0,
            radius: 15.0,
            last_mouse_x: None,
        }
    }
    pub fn reset(&mut self, position: Vec2) {
        self.position = position;
        self.angle = 0.0;
        self.last_mouse_x = None;
    }
    pub fn forward(&self) -> Vec2 {
        Vec2::new(self.angle.cos(), self.angle.sin())
    }
    pub fn read_input(&mut self, window: &Window, map: &GameMap, delta_seconds: f32) {
        let rotation_speed = 2.0 * delta_seconds;
        if window.is_key_down(Key::A) {
            self.angle -= rotation_speed;
        }
        if window.is_key_down(Key::D) {
            self.angle += rotation_speed;
        }
        if let Some((mouse_x, _)) = window.get_mouse_pos(MouseMode::Discard) {
            if let Some(previous_x) = self.last_mouse_x {
                self.angle += (mouse_x - previous_x) * 0.004;
            }
            self.last_mouse_x = Some(mouse_x);
        }
        let mut direction = 0.0;
        if window.is_key_down(Key::W) {
            direction += 1.0;
        }
        if window.is_key_down(Key::S) {
            direction -= 1.0;
        }
        if direction != 0.0 {
            self.move_with_collision(self.forward() * direction * 145.0 * delta_seconds, map);
        }
        self.angle = self.angle.rem_euclid(TAU);
    }
    pub fn move_with_collision(&mut self, delta: Vec2, map: &GameMap) {
        let candidate_x = Vec2::new(self.position.x + delta.x, self.position.y);
        if !self.collides(candidate_x, map) {
            self.position.x = candidate_x.x;
        }
        let candidate_y = Vec2::new(self.position.x, self.position.y + delta.y);
        if !self.collides(candidate_y, map) {
            self.position.y = candidate_y.y;
        }
    }
    fn collides(&self, position: Vec2, map: &GameMap) -> bool {
        let r = self.radius;
        let corners = [
            (position.x - r, position.y - r),
            (position.x + r, position.y - r),
            (position.x - r, position.y + r),
            (position.x + r, position.y + r),
        ];
        corners
            .iter()
            .any(|(x, y)| map.is_wall((*x / CELL_SIZE) as i32, (*y / CELL_SIZE) as i32))
    }
}
