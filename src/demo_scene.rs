// src/demo_scene.rs

use std::collections::HashMap;
use glam::{Mat4, Vec3};
use crate::engine_lib::scene_types::{
    Scene, HullBlueprint, BlueprintSide, HullInstance,
    HandlerConfig, SideHandlerTypeId,
    PortalConnectionInfo, PortalId,
    BlueprintId, InstanceId, SideIndex,
};

const CUBOID_BLUEPRINT_ID: BlueprintId = 0;
const ROOM1_INSTANCE_ID: InstanceId = 0;
const ROOM2_INSTANCE_ID: InstanceId = 1;

// Make these pub
pub const PORTAL_ID_FRONT: PortalId = 0;
pub const PORTAL_ID_BACK: PortalId = 1;
pub const PORTAL_ID_LEFT: PortalId = 2;
pub const PORTAL_ID_RIGHT: PortalId = 3;
pub const PORTAL_ID_TOP: PortalId = 4;
pub const PORTAL_ID_BOTTOM: PortalId = 5;

const CEILING_COLOR_CONF: HandlerConfig = HandlerConfig::StandardWall { color: [1.0, 0.0, 0.0, 1.0], texture_id: None };
const FLOOR_COLOR_CONF: HandlerConfig = HandlerConfig::StandardWall { color: [0.0, 1.0, 0.0, 1.0], texture_id: None };
const LEFT_WALL_COLOR_CONF: HandlerConfig = HandlerConfig::StandardWall { color: [1.0, 1.0, 1.0, 1.0], texture_id: None };
const RIGHT_WALL_COLOR_CONF: HandlerConfig = HandlerConfig::StandardWall { color: [0.5, 0.5, 0.5, 1.0], texture_id: None };
const FRONT_WALL_COLOR_BLUE_CONF: HandlerConfig = HandlerConfig::StandardWall { color: [0.3, 0.3, 0.8, 1.0], texture_id: None };
const BACK_WALL_YELLOW_CONF: HandlerConfig = HandlerConfig::StandardWall { color: [0.8, 0.8, 0.3, 1.0], texture_id: None };
const ORANGE_WALL_CONF: HandlerConfig = HandlerConfig::StandardWall {color: [0.9, 0.5, 0.2, 1.0], texture_id: None };

fn create_cuboid_room_blueprint() -> HullBlueprint {
    let half_size = 1.5;
    let vertices = vec![
        Vec3::new(-half_size, -half_size, -half_size), Vec3::new( half_size, -half_size, -half_size),
        Vec3::new( half_size,  half_size, -half_size), Vec3::new(-half_size,  half_size, -half_size),
        Vec3::new(-half_size, -half_size,  half_size), Vec3::new( half_size, -half_size,  half_size),
        Vec3::new( half_size,  half_size,  half_size), Vec3::new(-half_size,  half_size,  half_size),
    ];
    let sides = vec![
        // +Z face of blueprint (e.g. "front" if camera looks down -Z)
        // Normals point INWARD. So for +Z face, normal is (0,0,-1)
        BlueprintSide { vertex_indices: vec![4,5,6,7], local_normal: Vec3::new(0.0,0.0,-1.0), handler_type:SideHandlerTypeId::StandardWall, default_handler_config:FRONT_WALL_COLOR_BLUE_CONF.clone(), local_portal_id: Some(PORTAL_ID_FRONT) },
        // -Z face of blueprint ("back") -> Normal (0,0,1)
        BlueprintSide { vertex_indices: vec![1,0,3,2], local_normal: Vec3::new(0.0,0.0,1.0), handler_type:SideHandlerTypeId::StandardWall, default_handler_config:BACK_WALL_YELLOW_CONF.clone(), local_portal_id: Some(PORTAL_ID_BACK) },
        // -X face of blueprint ("left") -> Normal (1,0,0)
        BlueprintSide { vertex_indices: vec![0,4,7,3], local_normal: Vec3::new(1.0,0.0,0.0), handler_type:SideHandlerTypeId::StandardWall, default_handler_config:LEFT_WALL_COLOR_CONF.clone(), local_portal_id: Some(PORTAL_ID_LEFT) },
        // +X face of blueprint ("right") -> Normal (-1,0,0)
        BlueprintSide { vertex_indices: vec![5,1,2,6], local_normal: Vec3::new(-1.0,0.0,0.0), handler_type:SideHandlerTypeId::StandardWall, default_handler_config:RIGHT_WALL_COLOR_CONF.clone(), local_portal_id: Some(PORTAL_ID_RIGHT) },
        // +Y face of blueprint ("top", "ceiling") -> Normal (0,-1,0)
        BlueprintSide { vertex_indices: vec![7,6,2,3], local_normal: Vec3::new(0.0,-1.0,0.0), handler_type:SideHandlerTypeId::StandardWall, default_handler_config:CEILING_COLOR_CONF.clone(), local_portal_id: Some(PORTAL_ID_TOP) },
        // -Y face of blueprint ("bottom", "floor") -> Normal (0,1,0)
        BlueprintSide { vertex_indices: vec![0,1,5,4], local_normal: Vec3::new(0.0,1.0,0.0), handler_type:SideHandlerTypeId::StandardWall, default_handler_config:FLOOR_COLOR_CONF.clone(), local_portal_id: Some(PORTAL_ID_BOTTOM) },
    ];
    HullBlueprint { id: CUBOID_BLUEPRINT_ID, name: "CuboidRoomBlueprint_InwardNormals".to_string(), local_vertices: vertices, sides }
}

pub fn create_mvp_scene() -> Scene {
    let mut blueprints = HashMap::new();
    let cuboid_bp = create_cuboid_room_blueprint();
    blueprints.insert(cuboid_bp.id, cuboid_bp);

    let mut instances = HashMap::new();

    let mut room1_portal_connections = HashMap::new();
    let mut room1_side_configs = HashMap::new();
    // Room1's FRONT face (index 0, local_portal_id PORTAL_ID_FRONT) connects to Room2's BACK face (local_portal_id PORTAL_ID_BACK)
    room1_side_configs.insert(0 as SideIndex, HandlerConfig::StandardPortal { // Side 0 is +Z face (PORTAL_ID_FRONT)
        target_instance_id: ROOM2_INSTANCE_ID, target_portal_id: PORTAL_ID_BACK,
    });
    // PortalConnections might be redundant if handler configs are the primary source, but fill for completeness
    room1_portal_connections.insert(PORTAL_ID_FRONT, PortalConnectionInfo {
        target_instance_id: ROOM2_INSTANCE_ID, target_portal_id: PORTAL_ID_BACK,
    });
    let room1 = HullInstance {
        id: ROOM1_INSTANCE_ID, name: "Room1".to_string(), blueprint_id: CUBOID_BLUEPRINT_ID,
        initial_transform: Some(Mat4::from_translation(Vec3::new(0.0, 0.0, 0.0))),
        portal_connections: room1_portal_connections,
        instance_side_handler_configs: room1_side_configs,
    };
    instances.insert(room1.id, room1);

    let mut room2_portal_connections: HashMap<PortalId, PortalConnectionInfo> = HashMap::new();
    let mut room2_side_configs = HashMap::new();

    // Room2's BACK face (index 1, local_portal_id PORTAL_ID_BACK) connects back to Room1's FRONT face (local_portal_id PORTAL_ID_FRONT)
    room2_side_configs.insert(1 as SideIndex, HandlerConfig::StandardPortal { // Side 1 is -Z face (PORTAL_ID_BACK)
        target_instance_id: ROOM1_INSTANCE_ID,
        target_portal_id: PORTAL_ID_FRONT,
    });
    room2_portal_connections.insert(PORTAL_ID_BACK, PortalConnectionInfo { 
        target_instance_id: ROOM1_INSTANCE_ID,
        target_portal_id: PORTAL_ID_FRONT,
    });
    // Give Room2's front wall a distinct color so we know we're in room2
    room2_side_configs.insert(0 as SideIndex, ORANGE_WALL_CONF.clone()); // Side 0 (+Z face) of Room2

    let room2 = HullInstance {
        id: ROOM2_INSTANCE_ID, name: "Room2".to_string(), blueprint_id: CUBOID_BLUEPRINT_ID,
        initial_transform: None, // Positioned relative to Room1 via portal
        portal_connections: room2_portal_connections,
        instance_side_handler_configs: room2_side_configs,
    };
    instances.insert(room2.id, room2);

    // Initial camera position: in Room1, looking towards its +Z face (PORTAL_ID_FRONT)
    // which is the portal to Room2.
    // Camera default looks down its own -Z. To look at blueprint's +Z face (normal 0,0,-1),
    // camera's local +Z should align with blueprint's -Z. So RotY(PI).
    let initial_camera_position_in_room1 = Vec3::new(0.0, 0.0, -1.0); // Slightly back from center, inside Room1
    let initial_camera_yaw_rad = std::f32::consts::PI; // Yaw 180 deg to look at +Z face
    let initial_camera_pitch_rad = 0.0f32; 
    let rot_y = Mat4::from_rotation_y(initial_camera_yaw_rad);
    let rot_x = Mat4::from_rotation_x(initial_camera_pitch_rad);
    let initial_rotation = rot_y * rot_x;
    let initial_camera_transform = Mat4::from_translation(initial_camera_position_in_room1) * initial_rotation;

    Scene {
        blueprints, instances,
        active_camera_instance_id: ROOM1_INSTANCE_ID,
        active_camera_local_transform: initial_camera_transform,
    }
}