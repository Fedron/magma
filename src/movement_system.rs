use cgmath::prelude::*;
use winit::event::VirtualKeyCode;

use crate::{input::KeyboardInput, entity::Transform};

pub fn move_in_xz_plane(input: &KeyboardInput, transform: &mut Transform, dt: f32) {
    let mut rotation = cgmath::Vector3::new(0.0_f32, 0.0_f32, 0.0_f32);
    if input.is_key_pressed(VirtualKeyCode::Right) {
        rotation.y -= 1.0;
    }
    if input.is_key_pressed(VirtualKeyCode::Left) {
        rotation.y += 1.0;
    }
    if input.is_key_pressed(VirtualKeyCode::Up) {
        rotation.x -= 1.0;
    }
    if input.is_key_pressed(VirtualKeyCode::Down) {
        rotation.x += 1.0;
    }

    if rotation.dot(rotation) > std::f32::EPSILON {
        // 0.3 is the look speed TODO: make it customizable
        rotation = 5.0 * dt * rotation.normalize();
        transform.rotation.x += rotation.x;
        transform.rotation.y += rotation.y;
        transform.rotation.z += rotation.z;
    }

    transform.rotation.x = transform.rotation.x.clamp(-90.0, 90.0);
    transform.rotation.y = transform.rotation.y.clamp(-90.0, 90.0);

    let yaw = transform.rotation.y;
    let forward = cgmath::Vector3::new(yaw.sin(), 0.0, yaw.cos());
    let right = cgmath::Vector3::new(forward.z, 0.0, -forward.x);
    let up = cgmath::Vector3::new(0.0, -1.0, 0.0);

    let mut movement = cgmath::Vector3::new(0.0_f32, 0.0_f32, 0.0_f32);
    if input.is_key_pressed(VirtualKeyCode::W) {
        movement += forward;
    }if input.is_key_pressed(VirtualKeyCode::S) {
        movement -= forward;
    }if input.is_key_pressed(VirtualKeyCode::D) {
        movement += right;
    }if input.is_key_pressed(VirtualKeyCode::A) {
        movement -= right;
    }if input.is_key_pressed(VirtualKeyCode::Space) {
        movement += up;
    }if input.is_key_pressed(VirtualKeyCode::LShift) {
        movement -= up;
    }

    if movement.dot(movement) > std::f32::EPSILON {
        // 1 is the move speed TODO: make it customizable
        movement = 1.0 * dt * movement.normalize();
        transform.position.x += movement.x;
        transform.position.y += movement.y;
        transform.position.z += movement.z;
    }
}
