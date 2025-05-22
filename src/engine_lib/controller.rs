// src/engine_lib/controller.rs

use winit::{
    event::{WindowEvent, DeviceEvent, ElementState},
    keyboard::{KeyCode, PhysicalKey},
    window::{Window, CursorGrabMode},
};
use glam::{Mat4, Vec3, Vec4Swizzles};
use crate::engine_lib::scene_types::Scene;
use crate::engine_lib::scene_logic::update_camera_in_scene;


pub struct CameraController {
    pub camera_pos_delta: Vec3,
    pub camera_yaw_delta_keyboard: f32,
    pub camera_pitch_delta_keyboard: f32,

    pub mouse_dx_accum: f32,
    pub mouse_dy_accum: f32,

    current_yaw: f32,
    current_pitch: f32,

    pub mouse_sensitivity: f32,
    pub cursor_grabbed: bool,
}

impl CameraController {
    pub fn new(initial_yaw_rad: f32, initial_pitch_rad: f32, initial_grab: bool, sensitivity: f32) -> Self {
        Self {
            camera_pos_delta: Vec3::ZERO,
            camera_yaw_delta_keyboard: 0.0,
            camera_pitch_delta_keyboard: 0.0,
            mouse_dx_accum: 0.0,
            mouse_dy_accum: 0.0,
            current_yaw: initial_yaw_rad,
            current_pitch: initial_pitch_rad,
            mouse_sensitivity: sensitivity,
            cursor_grabbed: initial_grab,
        }
    }

    pub fn handle_window_event(&mut self, event: &WindowEvent, window: &Window) -> bool {
        match event {
            WindowEvent::KeyboardInput { event: key_event, .. } => {
                if key_event.state == ElementState::Pressed && key_event.physical_key == PhysicalKey::Code(KeyCode::Escape) {
                    self.toggle_cursor_grab(window);
                    return true;
                }
                let pressed = key_event.state == ElementState::Pressed;
                match key_event.physical_key {
                    PhysicalKey::Code(KeyCode::KeyW) => { self.camera_pos_delta.z = if pressed { -1.0 } else { 0.0 }; true }
                    PhysicalKey::Code(KeyCode::KeyS) => { self.camera_pos_delta.z = if pressed { 1.0 } else { 0.0 }; true }
                    PhysicalKey::Code(KeyCode::KeyA) => { self.camera_pos_delta.x = if pressed { -1.0 } else { 0.0 }; true }
                    PhysicalKey::Code(KeyCode::KeyD) => { self.camera_pos_delta.x = if pressed { 1.0 } else { 0.0 }; true }
                    PhysicalKey::Code(KeyCode::Space) => { self.camera_pos_delta.y = if pressed { 1.0 } else { 0.0 }; true }
                    PhysicalKey::Code(KeyCode::ShiftLeft) | PhysicalKey::Code(KeyCode::ControlLeft) => {
                        self.camera_pos_delta.y = if pressed { -1.0 } else { 0.0 }; true
                    }
                    PhysicalKey::Code(KeyCode::ArrowLeft) => { self.camera_yaw_delta_keyboard = if pressed { 1.0 } else { 0.0 }; true }
                    PhysicalKey::Code(KeyCode::ArrowRight) => { self.camera_yaw_delta_keyboard = if pressed { -1.0 } else { 0.0 }; true }
                    PhysicalKey::Code(KeyCode::ArrowUp) => { self.camera_pitch_delta_keyboard = if pressed { 1.0 } else { 0.0 }; true }
                    PhysicalKey::Code(KeyCode::ArrowDown) => { self.camera_pitch_delta_keyboard = if pressed { -1.0 } else { 0.0 }; true }
                    _ => false,
                }
            }
            WindowEvent::MouseInput { state, button, .. } => {
                if !self.cursor_grabbed && *state == ElementState::Pressed && *button == winit::event::MouseButton::Left {
                    self.grab_cursor(window, true);
                    return true;
                }
                false
            }
            WindowEvent::Focused(focused) => {
                if !*focused && self.cursor_grabbed {
                    self.grab_cursor(window, false);
                }
                false
            }
            _ => false,
        }
    }

    pub fn handle_device_event(&mut self, event: &DeviceEvent) {
        if !self.cursor_grabbed {
            self.mouse_dx_accum = 0.0;
            self.mouse_dy_accum = 0.0;
            return;
        }
        match event {
            DeviceEvent::MouseMotion { delta: (dx, dy) } => {
                self.mouse_dx_accum += *dx as f32;
                self.mouse_dy_accum += *dy as f32;
            }
            _ => {}
        }
    }

    pub fn toggle_cursor_grab(&mut self, window: &Window) {
        self.grab_cursor(window, !self.cursor_grabbed);
    }

    fn grab_cursor(&mut self, window: &Window, grab: bool) {
        if grab {
            if !self.cursor_grabbed {
                if window.set_cursor_grab(CursorGrabMode::Confined)
                    .or_else(|_e| window.set_cursor_grab(CursorGrabMode::Locked))
                    .is_ok() {
                    window.set_cursor_visible(false);
                    self.cursor_grabbed = true;
                } else {eprintln!("Could not grab cursor");}
            }
        } else {
            if self.cursor_grabbed {
                if window.set_cursor_grab(CursorGrabMode::None).is_ok() {
                    window.set_cursor_visible(true);
                    self.cursor_grabbed = false;
                    self.mouse_dx_accum = 0.0;
                    self.mouse_dy_accum = 0.0;
                } else {eprintln!("Could not ungrab cursor");}
            }
        }
    }

    pub fn apply_to_transform(
        &mut self,
        scene: &mut Scene, // Changed from &mut Mat4
        dt: f32
    ) {
        let move_speed = 3.0 * dt;
        let rot_speed_keyboard = 1.5 * dt;

        self.current_yaw -= self.mouse_dx_accum * self.mouse_sensitivity;
        self.current_yaw -= self.camera_yaw_delta_keyboard * rot_speed_keyboard;

        self.current_pitch -= self.mouse_dy_accum * self.mouse_sensitivity;
        self.current_pitch += self.camera_pitch_delta_keyboard * rot_speed_keyboard;

        self.mouse_dx_accum = 0.0;
        self.mouse_dy_accum = 0.0;

        let pitch_limit = std::f32::consts::FRAC_PI_2 - 0.01;
        self.current_pitch = self.current_pitch.clamp(-pitch_limit, pitch_limit);

        let rotation_y = Mat4::from_rotation_y(self.current_yaw);
        let rotation_x = Mat4::from_rotation_x(self.current_pitch);
        let new_rotation_matrix = rotation_y * rotation_x;

        let local_move_delta = Vec3::new(
            self.camera_pos_delta.x * move_speed,
            self.camera_pos_delta.y * move_speed,
            self.camera_pos_delta.z * move_speed,
        );
        
        let move_delta_in_host_space = new_rotation_matrix.transform_vector3(local_move_delta);
        
        let current_local_position = scene.active_camera_local_transform.w_axis.xyz();
        let potential_new_local_pos = current_local_position + move_delta_in_host_space;

        update_camera_in_scene(
            scene,
            potential_new_local_pos,
            new_rotation_matrix,
            dt
        );
    }
}