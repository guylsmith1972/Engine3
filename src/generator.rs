// src/generator.rs

use rand::Rng;
// crate::geometry now correctly refers to geometry module within the same library
use crate::geometry::{ConvexPolygon, Point2}; 

// ... rest of PolygonGenerator impl (no changes needed to logic)
pub struct PolygonGenerator;

impl PolygonGenerator {
    pub fn generate_convex_polygon(
        center_x: f32,
        center_y: f32,
        avg_radius: f32,
        num_vertices: usize,
    ) -> ConvexPolygon {
        let mut rng = rand::thread_rng();
        
        let mut angles = Vec::with_capacity(num_vertices);
        for i in 0..num_vertices {
            let base_angle = (i as f32) * 2.0 * std::f32::consts::PI / (num_vertices as f32);
            angles.push(base_angle);
        }
        
        let max_perturbation = std::f32::consts::PI / (num_vertices as f32) * 0.3; 
        
        for i in 0..num_vertices {
            let perturbation = rng.gen_range(-max_perturbation..max_perturbation);
            angles[i] += perturbation;
        }
        
        for i in 1..num_vertices {
            if angles[i] <= angles[i-1] {
                angles[i] = angles[i-1] + 0.01; 
            }
        }
        
        let mut points = Vec::with_capacity(num_vertices);
        
        let min_radius = avg_radius * 0.8;
        let max_radius = avg_radius * 1.2;
        
        for angle_rad in angles {
            let current_radius = rng.gen_range(min_radius..max_radius);
            
            points.push(Point2::new(
                center_x + current_radius * angle_rad.cos(),
                center_y + current_radius * angle_rad.sin(),
            ));
        }
        
        ConvexPolygon::from_points(&points)
    }
}