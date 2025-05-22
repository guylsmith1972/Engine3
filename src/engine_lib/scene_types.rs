// src/engine_lib/scene_types.rs

use crate::rendering_lib::geometry::ConvexPolygon;

// Type aliases for IDs
pub type BlueprintId = u32;
pub type InstanceId = u32;
pub type PortalId = u32;
pub type SideIndex = usize;

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Mat4 {
    pub data: [[f32; 4]; 4], // Row-major: data[row][column]
}

impl Mat4 {
    pub fn identity() -> Self {
        Self {
            data: [
                [1.0, 0.0, 0.0, 0.0],
                [0.0, 1.0, 0.0, 0.0],
                [0.0, 0.0, 1.0, 0.0],
                [0.0, 0.0, 0.0, 1.0],
            ],
        }
    }

    pub fn multiply(&self, other: &Mat4) -> Mat4 {
        let mut result = [[0.0f32; 4]; 4];
        for i in 0..4 {
            for j in 0..4 {
                for k in 0..4 {
                    result[i][j] += self.data[i][k] * other.data[k][j];
                }
            }
        }
        Mat4 { data: result }
    }

    pub fn transform_point(&self, point: &Point3) -> Point3 {
        let x = self.data[0][0] * point.x + self.data[0][1] * point.y + self.data[0][2] * point.z + self.data[0][3];
        let y = self.data[1][0] * point.x + self.data[1][1] * point.y + self.data[1][2] * point.z + self.data[1][3];
        let z = self.data[2][0] * point.x + self.data[2][1] * point.y + self.data[2][2] * point.z + self.data[2][3];
        let w = self.data[3][0] * point.x + self.data[3][1] * point.y + self.data[3][2] * point.z + self.data[3][3];

        if w.abs() < 1e-6 { // Avoid division by zero or very small w
            Point3::new(x, y, z) 
        } else {
            Point3::new(x / w, y / w, z / w)
        }
    }
    
    pub fn from_translation(translation: Point3) -> Self {
        Self {
            data: [
                [1.0, 0.0, 0.0, translation.x],
                [0.0, 1.0, 0.0, translation.y],
                [0.0, 0.0, 1.0, translation.z],
                [0.0, 0.0, 0.0, 1.0],
            ],
        }
    }

    pub fn from_rotation_y(angle_rad: f32) -> Self {
        let cos_a = angle_rad.cos();
        let sin_a = angle_rad.sin();
        Self {
            data: [
                [cos_a,  0.0, sin_a, 0.0],
                [0.0,    1.0, 0.0,   0.0],
                [-sin_a, 0.0, cos_a, 0.0],
                [0.0,    0.0, 0.0,   1.0],
            ],
        }
    }
    
    pub fn from_rotation_x(angle_rad: f32) -> Self {
        let cos_a = angle_rad.cos();
        let sin_a = angle_rad.sin();
        Self {
            data: [
                [1.0, 0.0,   0.0,    0.0],
                [0.0, cos_a, -sin_a, 0.0],
                [0.0, sin_a, cos_a,  0.0],
                [0.0, 0.0,   0.0,    1.0],
            ],
        }
    }
    
    pub fn from_rotation_z(angle_rad: f32) -> Self {
        let cos_a = angle_rad.cos();
        let sin_a = angle_rad.sin();
        Self {
            data: [
                [cos_a, -sin_a, 0.0, 0.0],
                [sin_a, cos_a,  0.0, 0.0],
                [0.0,   0.0,    1.0, 0.0],
                [0.0,   0.0,    0.0, 1.0],
            ],
        }
    }

    pub fn inverse(&self) -> Mat4 {
        // WARNING: This is a simplified inverse for orthonormal rotation + translation.
        // (Assumes M = T * R, so M_inv = R_inv * T_inv)
        let mut rot_inv_t = Mat4::identity();
        // Transpose the rotation part (upper 3x3 of self, which is R)
        for r_idx in 0..3 {
            for c_idx in 0..3 {
                rot_inv_t.data[r_idx][c_idx] = self.data[c_idx][r_idx];
            }
        }
        // Extract translation part T_vec from self
        let trans_vec = Point3::new(self.data[0][3], self.data[1][3], self.data[2][3]);
        // Calculate -R_inv * T_vec for the translation part of the final inverse matrix
        let inv_translation_component = rot_inv_t.transform_normal(&Point3::new(-trans_vec.x, -trans_vec.y, -trans_vec.z));
        
        rot_inv_t.data[0][3] = inv_translation_component.x;
        rot_inv_t.data[1][3] = inv_translation_component.y;
        rot_inv_t.data[2][3] = inv_translation_component.z;
        
        rot_inv_t
    }

    // Transforms a normal vector (direction only, no translation).
    // Assumes M's upper 3x3 is the rotation/scaling part.
    // For non-uniform scaling, inverse transpose of upper 3x3 is needed.
    // This simplified version is fine for rotation and uniform scaling.
    pub fn transform_normal(&self, normal: &Point3) -> Point3 {
        let x = self.data[0][0] * normal.x + self.data[0][1] * normal.y + self.data[0][2] * normal.z;
        let y = self.data[1][0] * normal.x + self.data[1][1] * normal.y + self.data[1][2] * normal.z;
        let z = self.data[2][0] * normal.x + self.data[2][1] * normal.y + self.data[2][2] * normal.z;
        Point3::new(x, y, z).normalize() 
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Point3 {
    pub x: f32,
    pub y: f32,
    pub z: f32,
}

impl Point3 {
    pub fn new(x: f32, y: f32, z: f32) -> Self { Self { x, y, z } }
    pub fn sub(&self, other: &Point3) -> Point3 { Point3::new(self.x - other.x, self.y - other.y, self.z - other.z) }
    pub fn dot(&self, other: &Point3) -> f32 { self.x * other.x + self.y * other.y + self.z * other.z }
    pub fn cross(&self, other: &Point3) -> Point3 {
        Point3::new(
            self.y * other.z - self.z * other.y,
            self.z * other.x - self.x * other.z,
            self.x * other.y - self.y * other.x,
        )
    }
    pub fn length(&self) -> f32 { (self.x * self.x + self.y * self.y + self.z * self.z).sqrt() }
    pub fn normalize(&self) -> Point3 {
        let l = self.length();
        if l.abs() < 1e-6 { Point3::new(0.0, 0.0, 0.0) } // Avoid division by zero
        else { Point3::new(self.x / l, self.y / l, self.z / l) }
    }
    pub fn add(&self, other: &Point3) -> Point3 { Point3::new(self.x + other.x, self.y + other.y, self.z + other.z) }
    pub fn mul_scalar(&self, scalar: f32) -> Point3 { Point3::new(self.x * scalar, self.y * scalar, self.z * scalar) }
}

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub enum SideHandlerTypeId {
    StandardWall,
    StandardPortal,
    Mirror,
    CameraDisplay,
    NonEuclideanPortal,
    TransparentWall,
}

#[derive(Clone, Debug)]
pub enum HandlerConfig {
    StandardWall { color: [f32; 4], texture_id: Option<String> },
    StandardPortal { target_instance_id: InstanceId, target_portal_id: PortalId },
    Mirror { recursion_limit: u8, surface_reflectivity: f32 },
    CameraDisplay { source_camera_id: String, refresh_rate: f32 },
    NonEuclideanPortal { target_instance_id: InstanceId, target_portal_id: PortalId, transform_params: String },
    TransparentWall { tint: [f32; 4], opacity: f32, ior: f32 },
    None,
}

impl HandlerConfig {
    pub fn get_intended_handler_type(&self) -> SideHandlerTypeId {
        match self {
            HandlerConfig::StandardWall { .. } => SideHandlerTypeId::StandardWall,
            HandlerConfig::StandardPortal { .. } => SideHandlerTypeId::StandardPortal,
            HandlerConfig::Mirror { .. } => SideHandlerTypeId::Mirror,
            HandlerConfig::CameraDisplay { .. } => SideHandlerTypeId::CameraDisplay,
            HandlerConfig::NonEuclideanPortal { .. } => SideHandlerTypeId::NonEuclideanPortal,
            HandlerConfig::TransparentWall { .. } => SideHandlerTypeId::TransparentWall,
            HandlerConfig::None => SideHandlerTypeId::StandardWall, 
        }
    }
}

#[derive(Clone, Debug)]
pub struct BlueprintSide {
    pub vertex_indices: Vec<usize>,
    pub local_normal: Point3, // Points TOWARDS THE INTERIOR of the blueprint
    pub handler_type: SideHandlerTypeId, 
    pub default_handler_config: HandlerConfig,
    pub local_portal_id: Option<PortalId>,
}

#[derive(Clone, Debug)]
pub struct HullBlueprint {
    pub id: BlueprintId,
    pub name: String,
    pub local_vertices: Vec<Point3>,
    pub sides: Vec<BlueprintSide>,
}

#[derive(Clone, Debug)]
pub struct PortalConnectionInfo {
    pub target_instance_id: InstanceId,
    pub target_portal_id: PortalId,
}

#[derive(Clone, Debug)]
pub struct HullInstance {
    pub id: InstanceId,
    pub name: String,
    pub blueprint_id: BlueprintId,
    pub initial_transform: Option<Mat4>,
    pub portal_connections: std::collections::HashMap<PortalId, PortalConnectionInfo>,
    pub instance_side_handler_configs: std::collections::HashMap<SideIndex, HandlerConfig>,
}

#[derive(Debug)]
pub struct Scene {
    pub blueprints: std::collections::HashMap<BlueprintId, HullBlueprint>,
    pub instances: std::collections::HashMap<InstanceId, HullInstance>,
    pub active_camera_instance_id: InstanceId,
    pub active_camera_local_transform: Mat4,
}

#[derive(Clone)]
pub struct TraversalState {
    pub current_instance_id: InstanceId,
    pub accumulated_transform: Mat4,
    pub screen_space_clip_polygon: ConvexPolygon,
    pub recursion_depth: u32,
}