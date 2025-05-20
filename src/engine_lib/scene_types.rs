// src/engine_lib/scene_types.rs

// Assuming `convex_polygon_intersection::geometry::ConvexPolygon` will be accessible
// via a dependency or another module in engine_lib or a shared core_lib.
// For now, let's assume it will be:
// use rendering_lib::geometry::ConvexPolygon; 
// OR use crate::rendering_lib::geometry::ConvexPolygon if engine_lib depends on rendering_lib
// OR use common_geometry::ConvexPolygon if you make a third geometry library.
// This needs to be resolved based on your final library structure.
// For this example, I'll use a placeholder that you'll need to fix.
use rendering_lib::geometry::ConvexPolygon; // Placeholder: Adjust this path!

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
    pub screen_space_clip_polygon: ConvexPolygon, // This type comes from geometry.rs
}