// src/camera.rs

use convex_polygon_intersection::geometry::Point2; // <--- Add this
use crate::scene::Point3;
// use std::f32::consts::PI;

#[derive(Debug)]
pub struct Camera {
    pub position: Point3,
    pub yaw: f32,   // Radians, rotation around Y axis
    pub pitch: f32, // Radians, rotation around X axis
    pub fov_y_rad: f32,
    pub znear: f32,
    pub zfar: f32,
}

impl Camera {
    pub fn new(
        position: Point3,
        yaw_deg: f32,
        pitch_deg: f32,
        fov_y_deg: f32,
        znear: f32,
        zfar: f32,
    ) -> Self {
        Self {
            position,
            yaw: yaw_deg.to_radians(),
            pitch: pitch_deg.to_radians(),
            fov_y_rad: fov_y_deg.to_radians(),
            znear,
            zfar,
        }
    }

    // Simplified projection: Transforms world point to 2D screen coordinates (0-width, 0-height)
    // This is a very basic implementation and doesn't handle all edge cases of a full 3D pipeline.
    pub fn project_to_screen(
        &self,
        world_point: &Point3,
        screen_width: f32,
        screen_height: f32,
    ) -> Option<crate::geometry::Point2> {
        // 1. Transform world point to camera space
        // For MVP, assume camera looks along its local -Z axis.
        // Yaw (around Y) and Pitch (around X)
        let cos_pitch = self.pitch.cos();
        let sin_pitch = self.pitch.sin();
        let cos_yaw = self.yaw.cos();
        let sin_yaw = self.yaw.sin();

        let mut p_cam = world_point.sub(&self.position);

        // Apply Yaw (around Y axis)
        let x_rotated_yaw = p_cam.x * cos_yaw - p_cam.z * sin_yaw;
        let z_rotated_yaw = p_cam.x * sin_yaw + p_cam.z * cos_yaw;
        p_cam.x = x_rotated_yaw;
        p_cam.z = z_rotated_yaw;

        // Apply Pitch (around X axis)
        let y_rotated_pitch = p_cam.y * cos_pitch - p_cam.z * sin_pitch;
        let z_rotated_pitch = p_cam.y * sin_pitch + p_cam.z * cos_pitch;
        p_cam.y = y_rotated_pitch;
        p_cam.z = z_rotated_pitch;


        // If point is behind the near plane (or too close)
        if p_cam.z >= -self.znear { // Using >= because we expect negative Z in front
            return None;
        }

        // 2. Perspective projection (camera space to NDC-like)
        // Camera looks down its local -Z axis.
        let aspect_ratio = screen_width / screen_height;
        let focal_length_y = 1.0 / (self.fov_y_rad / 2.0).tan();
        let focal_length_x = focal_length_y / aspect_ratio;

        // p_cam.z is negative for points in front
        let ndc_x = (p_cam.x * focal_length_x) / -p_cam.z;
        let ndc_y = (p_cam.y * focal_length_y) / -p_cam.z;

        // Check if outside NDC bounds (approximate frustum culling)
        if ndc_x < -1.1 || ndc_x > 1.1 || ndc_y < -1.1 || ndc_y > 1.1 {
            // A bit of margin to avoid issues at the exact edge with clipping
            // return None; // Disabled for MVP to see effects of clipping more clearly
        }


        // 3. Convert NDC to screen coordinates (0-width, 0-height)
        // Shader expects: normalized_x = (model.position.x / (width/2.0)) - 1.0
        //                   model.position.x = (normalized_x + 1.0) * width/2.0
        // Shader expects: normalized_y = 1.0 - (model.position.y / (height/2.0))
        //                   model.position.y = (1.0 - normalized_y) * height/2.0
        let screen_x = (ndc_x + 1.0) * 0.5 * screen_width;
        let screen_y = (1.0 - ndc_y) * 0.5 * screen_height; // Y is inverted

        Some(crate::geometry::Point2::new(screen_x, screen_y))
    }
}