// src/engine_lib/controller.rs

// Assuming Point3 will be part of scene_types in this library
use crate::scene_types::Point3; // Or if scene_types is a sibling module: use super::scene_types::Point3;
use crate::camera::Camera; // Assuming camera.rs is also in engine_lib

use winit::{
    event::{WindowEvent, DeviceEvent, ElementState, MouseScrollDelta},
    keyboard::{KeyCode, PhysicalKey},
    window::{Window, CursorGrabMode},
};

pub struct CameraController {
    // Input state previously in PolygonApp
    pub camera_pos_delta: Point3,
    pub camera_yaw_delta_accum: f32,   // For keyboard yaw
    pub camera_pitch_delta_accum: f32, // For keyboard pitch

    // Direct deltas for mouse, to be applied immediately to camera
    pub mouse_dx: f32,
    pub mouse_dy: f32,

    pub mouse_sensitivity: f32,
    pub cursor_grabbed: bool,
    // is_focused might still be useful here or managed by app and passed to controller methods
}

impl CameraController {
    pub fn new(initial_grab: bool, sensitivity: f32) -> Self {
        Self {
            camera_pos_delta: Point3::new(0.0, 0.0, 0.0),
            camera_yaw_delta_accum: 0.0,
            camera_pitch_delta_accum: 0.0,
            mouse_dx: 0.0,
            mouse_dy: 0.0,
            mouse_sensitivity: sensitivity,
            cursor_grabbed: initial_grab,
        }
    }

    // Process Winit window events for input
    pub fn handle_window_event(&mut self, event: &WindowEvent, window: &Window) -> bool { //
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
                    PhysicalKey::Code(KeyCode::ArrowLeft) => { self.camera_yaw_delta_accum = if pressed { 1.0 } else { 0.0 }; true } 
                    PhysicalKey::Code(KeyCode::ArrowRight) => { self.camera_yaw_delta_accum = if pressed { -1.0 } else { 0.0 }; true }
                    PhysicalKey::Code(KeyCode::ArrowUp) => { self.camera_pitch_delta_accum = if pressed { -1.0 } else { 0.0 }; true }
                    PhysicalKey::Code(KeyCode::ArrowDown) => { self.camera_pitch_delta_accum = if pressed { 1.0 } else { 0.0 }; true }
                    _ => false,
                }
            }
            WindowEvent::MouseInput { state, button, .. } => {
                // Assuming is_focused is checked by the caller (app.rs) before calling this
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
                false // App should still handle self.is_focused
            }
            _ => false,
        }
    }

    // Process Winit device events (raw mouse motion)
    pub fn handle_device_event(&mut self, event: &DeviceEvent) { //
        if !self.cursor_grabbed {
            self.mouse_dx = 0.0;
            self.mouse_dy = 0.0;
            return;
        }
        match event {
            DeviceEvent::MouseMotion { delta: (dx, dy) } => {
                self.mouse_dx = *dx as f32;
                self.mouse_dy = *dy as f32;
            }
            // MouseWheel could be handled here if needed
            _ => {}
        }
    }
    
    pub fn toggle_cursor_grab(&mut self, window: &Window) {
        self.grab_cursor(window, !self.cursor_grabbed);
    }
    
    // Extracted cursor grabbing logic
    fn grab_cursor(&mut self, window: &Window, grab: bool) { //
        if grab {
            if !self.cursor_grabbed { 
                window.set_cursor_grab(CursorGrabMode::Confined)
                    .or_else(|_e| window.set_cursor_grab(CursorGrabMode::Locked))
                    .unwrap_or_else(|e| eprintln!("Could not grab cursor: {:?}", e));
                window.set_cursor_visible(false);
                self.cursor_grabbed = true;
            }
        } else {
            if self.cursor_grabbed { 
                 window.set_cursor_grab(CursorGrabMode::None)
                    .unwrap_or_else(|e| eprintln!("Could not ungrab cursor: {:?}", e));
                window.set_cursor_visible(true);
                self.cursor_grabbed = false;
            }
        }
    }

    // Method to apply queued inputs to the camera
    // Call this once per frame in the app's update cycle
    pub fn update_camera(&mut self, camera: &mut Camera, dt: f32) { //
        let move_speed = 3.0 * dt; 
        let rot_speed = 1.5 * dt; 

        // Apply mouse look directly
        camera.yaw -= self.mouse_dx * self.mouse_sensitivity;
        camera.pitch += self.mouse_dy * self.mouse_sensitivity; 

        // Reset mouse deltas after applying them
        self.mouse_dx = 0.0;
        self.mouse_dy = 0.0;

        // Apply keyboard rotation
        camera.yaw += self.camera_yaw_delta_accum * rot_speed;
        camera.pitch += self.camera_pitch_delta_accum * rot_speed;
        
        // Clamp pitch
        camera.pitch = camera.pitch.clamp(
            -std::f32::consts::FRAC_PI_2 + 0.01, 
            std::f32::consts::FRAC_PI_2 - 0.01
        );

        // Apply movement based on current camera orientation (after rotation)
        let cos_pitch = camera.pitch.cos();
        let sin_pitch = camera.pitch.sin();
        let cos_yaw = camera.yaw.cos();
        let sin_yaw = camera.yaw.sin();

        let forward_dir = Point3::new(
            -sin_yaw * cos_pitch, 
            sin_pitch,            
            -cos_yaw * cos_pitch  
        ).normalize();
        
        let right_dir = Point3::new(-forward_dir.z, 0.0, forward_dir.x).normalize(); // Recalculate right based on new yaw