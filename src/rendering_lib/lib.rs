// rendering_lib/src/lib.rs
pub mod renderer;
pub mod shader;
pub mod vertex;
pub mod geometry; // For 2D screen-space operations used in portal culling
pub mod intersection; // For 2D screen-space operations used in portal culling

// Re-export key types if desired for easier use by the engine/app
pub use renderer::Renderer;
pub use vertex::Vertex;
pub use geometry::{Point2, ConvexPolygon, MAX_VERTICES};
pub use intersection::ConvexIntersection;
pub use shader::WGSL_SHADER_SOURCE;