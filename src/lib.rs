// src/lib.rs

pub mod geometry;
pub mod generator;
pub mod intersection;
pub mod vertex; // Assuming Vertex struct is core and might be used by geometry or other lib parts

// You could also re-export commonly used items for convenience if desired:
// pub use geometry::{ConvexPolygon, Point2, MAX_VERTICES};
// pub use generator::PolygonGenerator;
// pub use intersection::ConvexIntersection;
// pub use vertex::Vertex;