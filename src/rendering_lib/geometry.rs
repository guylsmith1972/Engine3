// src/geometry.rs

use bytemuck::{Pod, Zeroable};

pub const MAX_VERTICES: usize = 16;

#[repr(C)]
#[derive(Clone, Copy, Debug, Pod, Zeroable, PartialEq)]
pub struct Point2 {
    pub x: f32,
    pub y: f32,
}

impl Point2 {
    pub fn new(x: f32, y: f32) -> Self {
        Self { x, y }
    }

    pub fn dot(&self, other: &Point2) -> f32 {
        self.x * other.x + self.y * other.y
    }
}

#[derive(Clone, Debug)]
pub struct ConvexPolygon {
    vertices: [Point2; MAX_VERTICES], // Kept private for controlled access
    count: usize,
}

impl ConvexPolygon {
    pub fn new() -> Self {
        Self {
            vertices: [Point2::new(0.0, 0.0); MAX_VERTICES],
            count: 0,
        }
    }

    pub fn from_points(points: &[Point2]) -> Self {
        let mut polygon = Self::new();
        let num_to_copy = points.len().min(MAX_VERTICES);
        // Ensure we only copy if there are points to prevent panic on empty slice with [..num_to_copy]
        if num_to_copy > 0 {
             polygon.vertices[..num_to_copy].copy_from_slice(&points[..num_to_copy]);
        }
        polygon.count = num_to_copy;
        polygon
    }

    pub fn vertices(&self) -> &[Point2] {
        &self.vertices[..self.count]
    }
    
    pub fn count(&self) -> usize {
        self.count
    }

    pub fn set_count(&mut self, count: usize) {
        self.count = count.min(MAX_VERTICES);
    }

    pub fn copy_vertices_from_slice(&mut self, slice: &[Point2]) {
        let num_to_copy = slice.len().min(MAX_VERTICES);
        if num_to_copy > 0 {
            self.vertices[..num_to_copy].copy_from_slice(&slice[..num_to_copy]);
        } else {
            // If the slice is empty, ensure the polygon is also empty
        }
        self.count = num_to_copy; // Set count regardless, could be 0
    }

    pub fn area(&self) -> f32 {
        if self.count < 3 {
            return 0.0;
        }
        let mut area = 0.0;
        for i in 0..self.count {
            let j = (i + 1) % self.count;
            area += self.vertices[i].x * self.vertices[j].y;
            area -= self.vertices[j].x * self.vertices[i].y;
        }
        area.abs() / 2.0
    }
}