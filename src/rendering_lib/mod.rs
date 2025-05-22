// src/rendering_lib/mod.rs

pub mod renderer;
pub mod shader;
pub mod vertex;
pub mod geometry;
pub mod intersection;

pub use renderer::Renderer;
pub use vertex::Vertex;
pub use geometry::{Point2, ConvexPolygon, MAX_VERTICES};
pub use intersection::ConvexIntersection;
pub use shader::WGSL_SHADER_SOURCE;
// MAX_PORTAL_RECURSION_DEPTH is now in engine_lib::side_handler, so no need to export from here.