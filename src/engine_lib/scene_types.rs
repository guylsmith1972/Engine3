// src/engine_lib/scene_types.rs
use glam::{Mat4, Vec3};
use crate::rendering_lib::geometry::ConvexPolygon;

// Type aliases for IDs
pub type BlueprintId = u32;
pub type InstanceId = u32;
pub type PortalId = u32;
pub type SideIndex = usize;

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
            HandlerConfig::None => SideHandlerTypeId::StandardWall, // Default to wall if None
        }
    }
}

#[derive(Clone, Debug)]
pub struct BlueprintSide {
    pub vertex_indices: Vec<usize>,
    pub local_normal: Vec3,
    pub handler_type: SideHandlerTypeId,
    pub default_handler_config: HandlerConfig,
    pub local_portal_id: Option<PortalId>,
}

#[derive(Clone, Debug)]
pub struct HullBlueprint {
    pub id: BlueprintId,
    pub name: String,
    pub local_vertices: Vec<Vec3>,
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

// ADDED BoundaryCheckResult Enum
#[derive(Debug, Clone, PartialEq)]
pub enum BoundaryCheckResult {
    Inside,
    Collision {
        collided_side_index: SideIndex,
        collision_point: Vec3,
    },
    Traverse {
        crossed_side_index: SideIndex,
        target_instance_id: InstanceId,
        target_portal_id: PortalId,
    },
}