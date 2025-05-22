// src/demo_scene.rs

use std::collections::HashMap;
use crate::engine_lib::scene_types::{
    Scene, HullBlueprint, BlueprintSide, HullInstance,
    Point3, Mat4, HandlerConfig, SideHandlerTypeId,
    PortalConnectionInfo, PortalId,
    BlueprintId, InstanceId, SideIndex,
};

const CUBOID_BLUEPRINT_ID: BlueprintId = 0;
const ROOM1_INSTANCE_ID: InstanceId = 0;
const ROOM2_INSTANCE_ID: InstanceId = 1;

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
        Point3::new(-half_size, -half_size, -half_size), Point3::new( half_size, -half_size, -half_size),
        Point3::new( half_size,  half_size, -half_size), Point3::new(-half_size,  half_size, -half_size),
        Point3::new(-half_size, -half_size,  half_size), Point3::new( half_size, -half_size,  half_size),
        Point3::new( half_size,  half_size,  half_size), Point3::new(-half_size,  half_size,  half_size),
    ];
    let sides = vec![
        BlueprintSide { vertex_indices: vec![4,5,6,7], local_normal: Point3::new(0.0,0.0,-1.0), handler_type:SideHandlerTypeId::StandardWall, default_handler_config:FRONT_WALL_COLOR_BLUE_CONF.clone(), local_portal_id: Some(PORTAL_ID_FRONT) },
        BlueprintSide { vertex_indices: vec![1,0,3,2], local_normal: Point3::new(0.0,0.0,1.0), handler_type:SideHandlerTypeId::StandardWall, default_handler_config:BACK_WALL_YELLOW_CONF.clone(), local_portal_id: Some(PORTAL_ID_BACK) },
        BlueprintSide { vertex_indices: vec![0,4,7,3], local_normal: Point3::new(1.0,0.0,0.0), handler_type:SideHandlerTypeId::StandardWall, default_handler_config:LEFT_WALL_COLOR_CONF.clone(), local_portal_id: Some(PORTAL_ID_LEFT) },
        BlueprintSide { vertex_indices: vec![5,1,2,6], local_normal: Point3::new(-1.0,0.0,0.0), handler_type:SideHandlerTypeId::StandardWall, default_handler_config:RIGHT_WALL_COLOR_CONF.clone(), local_portal_id: Some(PORTAL_ID_RIGHT) },
        BlueprintSide { vertex_indices: vec![7,6,2,3], local_normal: Point3::new(0.0,-1.0,0.0), handler_type:SideHandlerTypeId::StandardWall, default_handler_config:CEILING_COLOR_CONF.clone(), local_portal_id: Some(PORTAL_ID_TOP) },
        BlueprintSide { vertex_indices: vec![0,1,5,4], local_normal: Point3::new(0.0,1.0,0.0), handler_type:SideHandlerTypeId::StandardWall, default_handler_config:FLOOR_COLOR_CONF.clone(), local_portal_id: Some(PORTAL_ID_BOTTOM) },
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
    room1_side_configs.insert(0 as SideIndex, HandlerConfig::StandardPortal { 
        target_instance_id: ROOM2_INSTANCE_ID, target_portal_id: PORTAL_ID_BACK, 
    });
    room1_portal_connections.insert(PORTAL_ID_FRONT, PortalConnectionInfo {
        target_instance_id: ROOM2_INSTANCE_ID, target_portal_id: PORTAL_ID_BACK,
    });
    let room1 = HullInstance {
        id: ROOM1_INSTANCE_ID, name: "Room1".to_string(), blueprint_id: CUBOID_BLUEPRINT_ID,
        initial_transform: Some(Mat4::from_translation(Point3::new(0.0, 0.0, 0.0))),
        portal_connections: room1_portal_connections, 
        instance_side_handler_configs: room1_side_configs,
    };
    instances.insert(room1.id, room1);

    // Room 2 connects back to Room 1
    let mut room2_portal_connections: HashMap<PortalId, PortalConnectionInfo> = HashMap::new(); // Explicit type
    let mut room2_side_configs = HashMap::new();
    
    room2_side_configs.insert(1 as SideIndex, HandlerConfig::StandardPortal { 
        target_instance_id: ROOM1_INSTANCE_ID,
        target_portal_id: PORTAL_ID_FRONT, 
    });
    room2_portal_connections.insert(PORTAL_ID_BACK, PortalConnectionInfo { // Add connection info
        target_instance_id: ROOM1_INSTANCE_ID,
        target_portal_id: PORTAL_ID_FRONT,
    });
    room2_side_configs.insert(0 as SideIndex, ORANGE_WALL_CONF.clone()); 

    let room2 = HullInstance {
        id: ROOM2_INSTANCE_ID, name: "Room2".to_string(), blueprint_id: CUBOID_BLUEPRINT_ID,
        initial_transform: None, 
        portal_connections: room2_portal_connections, 
        instance_side_handler_configs: room2_side_configs,
    };
    instances.insert(room2.id, room2);

    let initial_camera_position_in_room1 = Point3::new(0.0, 0.0, -1.0); 
    let initial_camera_yaw_rad = std::f32::consts::PI; 
    let initial_camera_pitch_rad = 0.0f32.to_radians();
    let rot_y = Mat4::from_rotation_y(initial_camera_yaw_rad);
    let rot_x = Mat4::from_rotation_x(initial_camera_pitch_rad);
    let initial_rotation = rot_y.multiply(&rot_x);
    let initial_camera_transform = Mat4::from_translation(initial_camera_position_in_room1).multiply(&initial_rotation);

    Scene {
        blueprints, instances,
        active_camera_instance_id: ROOM1_INSTANCE_ID,
        active_camera_local_transform: initial_camera_transform,
    }
}