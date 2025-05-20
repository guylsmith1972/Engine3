// src/scene.rs

use convex_polygon_intersection::geometry::{ConvexPolygon};


#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Point3 {
    pub x: f32,
    pub y: f32,
    pub z: f32,
}

impl Point3 {
    pub fn new(x: f32, y: f32, z: f32) -> Self {
        Self { x, y, z }
    }

    pub fn sub(&self, other: &Point3) -> Point3 {
        Point3::new(self.x - other.x, self.y - other.y, self.z - other.z)
    }

    pub fn dot(&self, other: &Point3) -> f32 {
        self.x * other.x + self.y * other.y + self.z * other.z
    }

    pub fn cross(&self, other: &Point3) -> Point3 { // Standard cross product
        Point3::new(
            self.y * other.z - self.z * other.y,
            self.z * other.x - self.x * other.z,
            self.x * other.y - self.y * other.x,
        )
    }
    
    pub fn length(&self) -> f32 {
        (self.x * self.x + self.y * self.y + self.z * self.z).sqrt()
    }

    pub fn normalize(&self) -> Point3 {
        let l = self.length();
        if l == 0.0 {
            Point3::new(0.0, 0.0, 0.0)
        } else {
            Point3::new(self.x / l, self.y / l, self.z / l)
        }
    }
}

#[derive(Clone, Debug)]
pub struct SceneSide {
    pub vertices_3d: Vec<Point3>, // In world coordinates, ordered CCW from outside
    pub normal: Point3,
    pub is_portal: bool,
    pub connected_hull_id: Option<usize>,
    pub color: [f32; 4],
}

impl SceneSide {
    pub fn calculate_normal(vertices: &[Point3]) -> Point3 {
        if vertices.len() < 3 {
            return Point3::new(0.0, 0.0, 1.0); // Default normal
        }
        let v0 = vertices[0];
        let v1 = vertices[1];
        let v2 = vertices[2];

        let edge1 = v1.sub(&v0);
        let edge2 = v2.sub(&v0);
        
        edge1.cross(&edge2).normalize()
    }
}

#[derive(Clone, Debug)]
pub struct Hull {
    pub id: usize,
    pub sides: Vec<SceneSide>,
    // pub center: Point3, // Optional: for quick culling or reference
}

#[derive(Debug)]
pub struct Scene {
    pub hulls: Vec<Hull>,
}

// Helper structure for portal traversal
#[derive(Clone)]
pub struct TraversalState {
    pub hull_id: usize,
    pub screen_space_clip_polygon: ConvexPolygon,
}


// Hardcoded MVP Scene
pub fn create_mvp_scene() -> Scene {
    let mut hulls = Vec::new();

    // Room 1 (Closer to Z=0)
    // A cube from (-1,-1,-1) to (1,1,1) relative to its center at (0,0,-2)
    let room1_center = Point3::new(0.0, 0.0, -3.0); // Move room further from camera
    let room1_half_size = 1.5;

    let r1_v = [
        Point3::new(room1_center.x - room1_half_size, room1_center.y - room1_half_size, room1_center.z - room1_half_size), // 0: LBF (Left Bottom Front)
        Point3::new(room1_center.x + room1_half_size, room1_center.y - room1_half_size, room1_center.z - room1_half_size), // 1: RBF
        Point3::new(room1_center.x + room1_half_size, room1_center.y + room1_half_size, room1_center.z - room1_half_size), // 2: RTF
        Point3::new(room1_center.x - room1_half_size, room1_center.y + room1_half_size, room1_center.z - room1_half_size), // 3: LTF
        Point3::new(room1_center.x - room1_half_size, room1_center.y - room1_half_size, room1_center.z + room1_half_size), // 4: LBB (Left Bottom Back)
        Point3::new(room1_center.x + room1_half_size, room1_center.y - room1_half_size, room1_center.z + room1_half_size), // 5: RBB
        Point3::new(room1_center.x + room1_half_size, room1_center.y + room1_half_size, room1_center.z + room1_half_size), // 6: RTB
        Point3::new(room1_center.x - room1_half_size, room1_center.y + room1_half_size, room1_center.z + room1_half_size), // 7: LTB
    ];
    
    let room1_sides = vec![
        // Front face (Portal to Room 2) - Vertices ordered CCW from POV outside room1 looking in
        SceneSide { // This is the portal FROM room 1 TO room 2
            vertices_3d: vec![r1_v[4], r1_v[5], r1_v[6], r1_v[7]], // Back face of cube1, becomes portal
            normal: Point3::new(0.0, 0.0, 1.0), // Normal pointing outwards from room1 (along +Z)
            is_portal: true,
            connected_hull_id: Some(1), // Connects to Hull ID 1 (Room 2)
            color: [0.0, 0.0, 0.0, 0.0], // Not rendered itself
        },
        // Left Wall
        SceneSide {
            vertices_3d: vec![r1_v[0], r1_v[4], r1_v[7], r1_v[3]], // Left face: LBF, LBB, LTB, LTF
            normal: Point3::new(-1.0, 0.0, 0.0), // Points -X
            is_portal: false, connected_hull_id: None, color: [0.8, 0.2, 0.2, 0.7], // Red-ish
        },
        // Right Wall
        SceneSide {
            vertices_3d: vec![r1_v[5], r1_v[1], r1_v[2], r1_v[6]], // Right face: RBB, RBF, RTF, RTB
            normal: Point3::new(1.0, 0.0, 0.0), // Points +X
            is_portal: false, connected_hull_id: None, color: [0.2, 0.8, 0.2, 0.7], // Green-ish
        },
        // Bottom Wall
        SceneSide {
            vertices_3d: vec![r1_v[4], r1_v[0], r1_v[1], r1_v[5]], // Bottom face: LBB, LBF, RBF, RBB
            normal: Point3::new(0.0, -1.0, 0.0), // Points -Y
            is_portal: false, connected_hull_id: None, color: [0.2, 0.2, 0.8, 0.7], // Blue-ish
        },
        // Top Wall
        SceneSide {
            vertices_3d: vec![r1_v[3], r1_v[7], r1_v[6], r1_v[2]], // Top face: LTF, LTB, RTB, RTF
            normal: Point3::new(0.0, 1.0, 0.0), // Points +Y
            is_portal: false, connected_hull_id: None, color: [0.8, 0.8, 0.2, 0.7], // Yellow-ish
        },
         // Back Wall (closest to camera, should be visible if camera is outside)
        SceneSide {
            vertices_3d: vec![r1_v[1],r1_v[0],r1_v[3],r1_v[2]], // Original Front face of cube1
            normal: Point3::new(0.0, 0.0, -1.0), // Normal pointing outwards from room1 (along -Z)
            is_portal: false,
            connected_hull_id: None,
            color: [0.5, 0.5, 0.5, 0.7], // Grey
        },
    ];
    hulls.push(Hull { id: 0, sides: room1_sides });


    // Room 2 (Further from Z=0, behind Room 1's portal)
    let room2_center = Point3::new(0.0, 0.0, room1_center.z + room1_half_size * 2.0 + 0.1); // Place it behind room1, aligned with portal
    let room2_half_size = 1.5;
    let r2_v = [
        Point3::new(room2_center.x - room2_half_size, room2_center.y - room2_half_size, room2_center.z - room2_half_size), // 0: LBF
        Point3::new(room2_center.x + room2_half_size, room2_center.y - room2_half_size, room2_center.z - room2_half_size), // 1: RBF
        Point3::new(room2_center.x + room2_half_size, room2_center.y + room2_half_size, room2_center.z - room2_half_size), // 2: RTF
        Point3::new(room2_center.x - room2_half_size, room2_center.y + room2_half_size, room2_center.z - room2_half_size), // 3: LTF
        Point3::new(room2_center.x - room2_half_size, room2_center.y - room2_half_size, room2_center.z + room2_half_size), // 4: LBB
        Point3::new(room2_center.x + room2_half_size, room2_center.y - room2_half_size, room2_center.z + room2_half_size), // 5: RBB
        Point3::new(room2_center.x + room2_half_size, room2_center.y + room2_half_size, room2_center.z + room2_half_size), // 6: RTB
        Point3::new(room2_center.x - room2_half_size, room2_center.y + room2_half_size, room2_center.z + room2_half_size), // 7: LTB
    ];

    let room2_sides = vec![
        // Portal Wall (Front face of room2, connects back to Room 1)
        // This is the portal FROM room 2 TO room 1
        SceneSide {
            vertices_3d: vec![r2_v[3], r2_v[2], r2_v[1], r2_v[0]], // Front face of cube2, CCW from outside
            normal: Point3::new(0.0, 0.0, -1.0), // Normal pointing outwards from room2 (along -Z)
            is_portal: true,
            connected_hull_id: Some(0), // Connects to Hull ID 0 (Room 1)
            color: [0.0, 0.0, 0.0, 0.0], // Not rendered
        },
        // Back Wall
        SceneSide {
            vertices_3d: vec![r2_v[5], r2_v[4], r2_v[7], r2_v[6]], // Back face of cube2
            normal: Point3::new(0.0, 0.0, 1.0), // Normal pointing outwards from room2 (along +Z)
            is_portal: false, connected_hull_id: None, color: [0.9, 0.5, 0.2, 0.8], // Orange
        },
        // Left Wall
        SceneSide {
            vertices_3d: vec![r2_v[0], r2_v[4], r2_v[7], r2_v[3]],
            normal: Point3::new(-1.0, 0.0, 0.0),
            is_portal: false, connected_hull_id: None, color: [0.2, 0.9, 0.5, 0.8], // Teal
        },
        // Right Wall
        SceneSide {
            vertices_3d: vec![r2_v[5], r2_v[1], r2_v[2], r2_v[6]],
            normal: Point3::new(1.0, 0.0, 0.0),
            is_portal: false, connected_hull_id: None, color: [0.5, 0.2, 0.9, 0.8], // Purple
        },
         // Bottom Wall
        SceneSide {
            vertices_3d: vec![r2_v[4], r2_v[0], r2_v[1], r2_v[5]],
            normal: Point3::new(0.0, -1.0, 0.0),
            is_portal: false, connected_hull_id: None, color: [0.1, 0.4, 0.4, 0.7], 
        },
        // Top Wall
        SceneSide {
            vertices_3d: vec![r2_v[3], r2_v[7], r2_v[6], r2_v[2]],
            normal: Point3::new(0.0, 1.0, 0.0),
            is_portal: false, connected_hull_id: None, color: [0.4, 0.1, 0.1, 0.7],
        },
    ];
    hulls.push(Hull { id: 1, sides: room2_sides });


    Scene { hulls }
}