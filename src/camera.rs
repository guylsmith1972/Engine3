// src/camera.rs

use convex_polygon_intersection::geometry::Point2; // Ensure this line is present
use crate::scene::Point3;
// PI is not used in the current version of camera.rs, so the 'use' for it can be removed for now
// use std::f32::consts::PI;

#[derive(Debug)]
pub struct Camera {
    pub position: Point3,
    pub yaw: f32,
    pub pitch: f32,
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

    pub fn project_to_screen(
        &self,
        world_point: &Point3,
        screen_width: f32,
        screen_height: f32,
    ) -> Option<Point2> { // **** CORRECTED: Was crate::geometry::Point2 ****
        // 1. Transform world point to camera space
        let cos_pitch = self.pitch.cos();
        let sin_pitch = self.pitch.sin();
        let cos_yaw = self.yaw.cos();
        let sin_yaw = self.yaw.sin();

        let mut p_cam = world_point.sub(&self.position);

        let x_rotated_yaw = p_cam.x * cos_yaw - p_cam.z * sin_yaw;
        let z_rotated_yaw = p_cam.x * sin_yaw + p_cam.z * cos_yaw;
        p_cam.x = x_rotated_yaw;
        p_cam.z = z_rotated_yaw;

        let y_rotated_pitch = p_cam.y * cos_pitch - p_cam.z * sin_pitch;
        let z_rotated_pitch = p_cam.y * sin_pitch + p_cam.z * cos_pitch;
        p_cam.y = y_rotated_pitch;
        p_cam.z = z_rotated_pitch;

        if p_cam.z >= -self.znear {
            return None;
        }

        let aspect_ratio = screen_width / screen_height;
        let focal_length_y = 1.0 / (self.fov_y_rad / 2.0).tan();
        let focal_length_x = focal_length_y / aspect_ratio;

        let ndc_x = (p_cam.x * focal_length_x) / -p_cam.z;
        let ndc_y = (p_cam.y * focal_length_y) / -p_cam.z;

        let screen_x = (ndc_x + 1.0) * 0.5 * screen_width;
        let screen_y = (1.0 - ndc_y) * 0.5 * screen_height;

        Some(Point2::new(screen_x, screen_y)) // **** CORRECTED: Was crate::geometry::Point2::new ****
    }
}