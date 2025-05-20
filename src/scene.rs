// src/scene.rs

use convex_polygon_intersection::geometry::{ConvexPolygon, Point2};

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
        if l == 0.0 { Point3::new(0.0, 0.0, 0.0) } 
        else { Point3::new(self.x / l, self.y / l, self.z / l) }
    }
    pub fn add(&self, other: &Point3) -> Point3 { Point3::new(self.x + other.x, self.y + other.y, self.z + other.z) }
    pub fn mul_scalar(&self, scalar: f32) -> Point3 { Point3::new(self.x * scalar, self.y * scalar, self.z * scalar) }
}

#[derive(Clone, Debug)]
pub struct SceneSide {
    pub vertices_3d: Vec<Point3>,
    pub normal: Point3,
    pub is_portal: bool,
    pub connected_hull_id: Option<usize>,
    pub color: [f32; 4],
}

impl SceneSide {
    pub fn calculate_normal(vertices: &[Point3]) -> Point3 {
        if vertices.len() < 3 { return Point3::new(0.0, 0.0, 1.0); }
        let v0 = vertices[0]; let v1 = vertices[1]; let v2 = vertices[2];
        let edge1 = v1.sub(&v0); let edge2 = v2.sub(&v0);
        edge1.cross(&edge2).normalize()
    }
}

#[derive(Clone, Debug)]
pub struct Hull {
    pub id: usize,
    pub sides: Vec<SceneSide>,
}

#[derive(Debug)]
pub struct Scene {
    pub hulls: Vec<Hull>,
}

#[derive(Clone)]
pub struct TraversalState {
    pub hull_id: usize,
    pub screen_space_clip_polygon: ConvexPolygon,
}

pub fn create_mvp_scene() -> Scene {
    let mut hulls = Vec::new();

    const CEILING_COLOR: [f32; 4] = [1.0, 0.0, 0.0, 1.0]; // Red
    const FLOOR_COLOR:   [f32; 4] = [0.0, 1.0, 0.0, 1.0]; // Green
    const LEFT_WALL_COLOR: [f32; 4] = [1.0, 1.0, 1.0, 1.0]; // White
    const RIGHT_WALL_COLOR: [f32; 4] = [0.5, 0.5, 0.5, 1.0]; // Dark Gray (using 0.5 to distinguish from other grays)
    const ALPHA: f32 = 1.0;

    let room1_center = Point3::new(0.0, 0.0, -3.0);
    let room1_half_size = 1.5;
    let r1_v = [ /* ... vertex definitions remain the same ... */
        Point3::new(room1_center.x - room1_half_size, room1_center.y - room1_half_size, room1_center.z - room1_half_size),
        Point3::new(room1_center.x + room1_half_size, room1_center.y - room1_half_size, room1_center.z - room1_half_size),
        Point3::new(room1_center.x + room1_half_size, room1_center.y + room1_half_size, room1_center.z - room1_half_size),
        Point3::new(room1_center.x - room1_half_size, room1_center.y + room1_half_size, room1_center.z - room1_half_size),
        Point3::new(room1_center.x - room1_half_size, room1_center.y - room1_half_size, room1_center.z + room1_half_size),
        Point3::new(room1_center.x + room1_half_size, room1_center.y - room1_half_size, room1_center.z + room1_half_size),
        Point3::new(room1_center.x + room1_half_size, room1_center.y + room1_half_size, room1_center.z + room1_half_size),
        Point3::new(room1_center.x - room1_half_size, room1_center.y + room1_half_size, room1_center.z + room1_half_size),
    ];
    
    let room1_sides = vec![
        SceneSide { // Portal
            vertices_3d: vec![r1_v[4], r1_v[5], r1_v[6], r1_v[7]], 
            normal: Point3::new(0.0, 0.0, 1.0), is_portal: true, connected_hull_id: Some(1), 
            color: [0.0, 0.0, 0.0, 0.0],
        },
        SceneSide { // Left Wall (-X side of room)
            vertices_3d: vec![r1_v[0], r1_v[4], r1_v[7], r1_v[3]], 
            normal: Point3::new(-1.0, 0.0, 0.0), is_portal: false, connected_hull_id: None, 
            color: LEFT_WALL_COLOR, // **** MODIFIED ****
        },
        SceneSide { // Right Wall (+X side of room)
            vertices_3d: vec![r1_v[5], r1_v[1], r1_v[2], r1_v[6]], 
            normal: Point3::new(1.0, 0.0, 0.0), is_portal: false, connected_hull_id: None, 
            color: RIGHT_WALL_COLOR, // **** MODIFIED ****
        },
        SceneSide { // Bottom Wall (Floor)
            vertices_3d: vec![r1_v[4], r1_v[0], r1_v[1], r1_v[5]], 
            normal: Point3::new(0.0, -1.0, 0.0), is_portal: false, connected_hull_id: None, 
            color: FLOOR_COLOR,
        },
        SceneSide { // Top Wall (Ceiling)
            vertices_3d: vec![r1_v[3], r1_v[7], r1_v[6], r1_v[2]], 
            normal: Point3::new(0.0, 1.0, 0.0), is_portal: false, connected_hull_id: None, 
            color: CEILING_COLOR,
        },
        SceneSide { // Back Wall
            vertices_3d: vec![r1_v[1],r1_v[0],r1_v[3],r1_v[2]], 
            normal: Point3::new(0.0, 0.0, -1.0), is_portal: false, connected_hull_id: None, 
            color: [0.3, 0.3, 0.3, ALPHA], // Darker Grey for this back wall
        },
    ];
    hulls.push(Hull { id: 0, sides: room1_sides });

    let room2_center = Point3::new(0.0, 0.0, room1_center.z + room1_half_size * 2.0 + 0.1);
    let room2_half_size = 1.5;
    let r2_v = [ /* ... vertex definitions remain the same ... */
        Point3::new(room2_center.x - room2_half_size, room2_center.y - room2_half_size, room2_center.z - room2_half_size),
        Point3::new(room2_center.x + room2_half_size, room2_center.y - room2_half_size, room2_center.z - room2_half_size),
        Point3::new(room2_center.x + room2_half_size, room2_center.y + room2_half_size, room2_center.z - room2_half_size),
        Point3::new(room2_center.x - room2_half_size, room2_center.y + room2_half_size, room2_center.z - room2_half_size),
        Point3::new(room2_center.x - room2_half_size, room2_center.y - room2_half_size, room2_center.z + room2_half_size),
        Point3::new(room2_center.x + room2_half_size, room2_center.y - room2_half_size, room2_center.z + room2_half_size),
        Point3::new(room2_center.x + room2_half_size, room2_center.y + room2_half_size, room2_center.z + room2_half_size),
        Point3::new(room2_center.x - room2_half_size, room2_center.y + room2_half_size, room2_center.z + room2_half_size),
    ];

    let room2_sides = vec![
        SceneSide { // Portal Wall
            vertices_3d: vec![r2_v[3], r2_v[2], r2_v[1], r2_v[0]],
            normal: Point3::new(0.0, 0.0, -1.0), is_portal: true, connected_hull_id: Some(0),
            color: [0.0, 0.0, 0.0, 0.0],
        },
        SceneSide { // Back Wall
            vertices_3d: vec![r2_v[5], r2_v[4], r2_v[7], r2_v[6]],
            normal: Point3::new(0.0, 0.0, 1.0), is_portal: false, connected_hull_id: None, 
            color: [0.9, 0.5, 0.2, ALPHA], // Orange
        },
        SceneSide { // Left Wall (-X side of room)
            vertices_3d: vec![r2_v[0], r2_v[4], r2_v[7], r2_v[3]],
            normal: Point3::new(-1.0, 0.0, 0.0), is_portal: false, connected_hull_id: None, 
            color: LEFT_WALL_COLOR, // **** MODIFIED ****
        },
        SceneSide { // Right Wall (+X side of room)
            vertices_3d: vec![r2_v[5], r2_v[1], r2_v[2], r2_v[6]],
            normal: Point3::new(1.0, 0.0, 0.0), is_portal: false, connected_hull_id: None, 
            color: RIGHT_WALL_COLOR, // **** MODIFIED ****
        },
         SceneSide { // Bottom Wall (Floor)
            vertices_3d: vec![r2_v[4], r2_v[0], r2_v[1], r2_v[5]],
            normal: Point3::new(0.0, -1.0, 0.0), is_portal: false, connected_hull_id: None, 
            color: FLOOR_COLOR,
        },
        SceneSide { // Top Wall (Ceiling)
            vertices_3d: vec![r2_v[3], r2_v[7], r2_v[6], r2_v[2]],
            normal: Point3::new(0.0, 1.0, 0.0), is_portal: false, connected_hull_id: None, 
            color: CEILING_COLOR,
        },
    ];
    hulls.push(Hull { id: 1, sides: room2_sides });

    Scene { hulls }
}