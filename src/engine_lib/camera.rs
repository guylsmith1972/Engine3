// src/camera.rs

use convex_polygon_intersection::geometry::Point2;
use crate::scene::Point3;

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

    pub fn transform_to_camera_space(&self, world_point: &Point3) -> Point3 {
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

        p_cam
    }

    pub fn project_camera_space_to_screen(
        &self,
        p_cam: &Point3,
        screen_width: f32,
        screen_height: f32,
    ) -> Option<Point2> {
        // Near plane check (camera looks down its local -Z axis, points in front have p_cam.z < 0)
        // Cull if p_cam.z is *behind or on* the near plane from the eye's perspective.
        // We want to allow points ON the near plane if our clipper put them there.
        // A point is "behind" if its z value in camera space is > -self.znear
        // (i.e., less negative than -self.znear, or positive).
        //
        // **** MODIFIED CHECK ****
        // Old: if p_cam.z >= -self.znear 
        // New: Only discard if strictly behind the near plane.
        //      Points on the near plane (p_cam.z == -self.znear) should be projectable.
        if p_cam.z > -self.znear {  // Cull if z is greater (less negative / more positive) than -znear
            return None;
        }
        // Optional: Add a very small epsilon if floating point issues persist with exact equality:
        // if p_cam.z > -self.znear + 1e-6 { return None; }


        if p_cam.z < -self.zfar { // Far plane check
            return None;
        }
        
        // Avoid division by zero or by a number very close to zero if p_cam.z is -self.znear
        // and self.znear is extremely small, or if -p_cam.z itself is near zero.
        // The check `p_cam.z > -self.znear` should already prevent `p_cam.z` from being positive or zero.
        // It ensures `-p_cam.z` is at least `self.znear` (which is 0.1).
        if -p_cam.z < 1e-6 { // Effectively checks if p_cam.z is too close to 0 from negative side
            return None;
        }


        let aspect_ratio = screen_width / screen_height;
        let focal_length_y = 1.0 / (self.fov_y_rad / 2.0).tan();
        let focal_length_x = focal_length_y / aspect_ratio;

        let ndc_x = (p_cam.x * focal_length_x) / -p_cam.z;
        let ndc_y = (p_cam.y * focal_length_y) / -p_cam.z;

        let screen_x = (ndc_x + 1.0) * 0.5 * screen_width;
        let screen_y = (1.0 - ndc_y) * 0.5 * screen_height;

        Some(Point2::new(screen_x, screen_y))
    }

    pub fn project_to_screen(
        &self,
        world_point: &Point3,
        screen_width: f32,
        screen_height: f32,
    ) -> Option<Point2> {
        let p_cam = self.transform_to_camera_space(world_point);
        self.project_camera_space_to_screen(&p_cam, screen_width, screen_height)
    }
}