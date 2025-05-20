// src/shader.rs

pub const WGSL_SHADER_SOURCE: &str = r#"
struct ScreenDimensions {
    width: f32,
    height: f32,
    //_padding1: f32, // Add padding if needed for 16-byte alignment for webgl2
    //_padding2: f32,
}

@group(0) @binding(0)
var<uniform> screen: ScreenDimensions;

struct VertexInput {
    @location(0) position: vec2<f32>, // These are screen-space coordinates
    @location(1) color: vec4<f32>,
}

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>, // Output to Normalized Device Coordinates
    @location(0) color: vec4<f32>,
}

@vertex
fn vs_main(model: VertexInput) -> VertexOutput {
    var out: VertexOutput;
    out.color = model.color;

    // Convert screen coordinates (model.position) to NDC
    // model.position.x is in [0, screen.width]
    // model.position.y is in [0, screen.height] (origin top-left)
    
    let normalized_x = (model.position.x / (screen.width / 2.0)) - 1.0;
    // For normalized_y, typical NDC has +Y up. Screen coords often have +Y down.
    // If model.position.y is 0 at top and screen.height at bottom:
    // (model.position.y / (screen.height / 2.0)) gives [0, 2]
    // 1.0 - ... maps [0, 2] to [1.0, -1.0] (correct for NDC Y up)
    let normalized_y = 1.0 - (model.position.y / (screen.height / 2.0)); 
    
    out.clip_position = vec4<f32>(normalized_x, normalized_y, 0.0, 1.0);
    
    return out;
}

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    return in.color;
}
"#;