// src/engine_lib/camera.rs

use crate::engine_lib::scene_types::{Point3, Mat4};
use crate::rendering_lib::geometry::Point2;

#[derive(Debug)]
pub struct Camera {
    pub fov_y_rad: f32,
    pub znear: f32,
    pub zfar: f32,
}

impl Camera {
    pub fn new(
        fov_y_deg: f32,
        znear: f32,
        zfar: f32,
    ) -> Self {
        Self {
            fov_y_rad: fov_y_deg.to_radians(),
            znear,
            zfar,
        }
    }

    // Constructs the view matrix that transforms points from the
    // camera's host hull's blueprint space into the camera's view space.
    // `camera_pose_in_host_hull` is the transform from CamLocal -> HostHullBlueprint.
    // The view matrix is its inverse: HostHullBlueprint -> CamLocal.
    pub fn get_view_matrix_from_host_hull(&self, camera_pose_in_host_hull: &Mat4) -> Mat4 {
        camera_pose_in_host_hull.inverse()
    }

    // Projects points that are ALREADY in camera view space to screen space.
    pub fn project_camera_space_to_screen_direct(
        &self,
        p_cam: &Point3, // Point in camera view space
        screen_width: f32,
        screen_height: f32,
    ) -> Option<Point2> {
        if p_cam.z > -self.znear + 1e-6 { // Cull if z is greater (less negative / more positive) than -znear
            return None;
        }
        if p_cam.z < -self.zfar { // Far plane check
            return None;
        }
        if -p_cam.z < 1e-6 { // Avoid division by zero if p_cam.z is too close to 0 from negative side
            return None;
        }

        let aspect_ratio = screen_width / screen_height;
        let focal_length_y = 1.0 / (self.fov_y_rad / 2.0).tan();
        let focal_length_x = focal_length_y / aspect_ratio;

        let ndc_x = (p_cam.x * focal_length_x) / -p_cam.z;
        let ndc_y = (p_cam.y * focal_length_y) / -p_cam.z;

        let screen_x = (ndc_x + 1.0) * 0.5 * screen_width;
        let screen_y = (1.0 - ndc_y) * 0.5 * screen_height; // Invert Y for screen space

        Some(Point2::new(screen_x, screen_y))
    }
}