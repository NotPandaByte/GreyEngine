// ============================================================================
// Uniforms
// ============================================================================

struct CameraUniform {
    view_proj: mat4x4<f32>,
}

@group(0) @binding(0)
var<uniform> camera: CameraUniform;

// ============================================================================
// 2D Vertex Shader
// ============================================================================

struct VertexInput2D {
    @location(0) position: vec2<f32>,
    @location(1) uv: vec2<f32>,
    @location(2) color: vec4<f32>,
}

struct VertexOutput2D {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) uv: vec2<f32>,
    @location(1) color: vec4<f32>,
}

@vertex
fn vs_main_2d(in: VertexInput2D) -> VertexOutput2D {
    var out: VertexOutput2D;
    out.clip_position = camera.view_proj * vec4<f32>(in.position, 0.0, 1.0);
    out.uv = in.uv;
    out.color = in.color;
    return out;
}

// ============================================================================
// 2D Fragment Shader (with texture)
// ============================================================================

@group(1) @binding(0)
var t_diffuse: texture_2d<f32>;
@group(1) @binding(1)
var s_diffuse: sampler;

@fragment
fn fs_main_2d(in: VertexOutput2D) -> @location(0) vec4<f32> {
    let tex_color = textureSample(t_diffuse, s_diffuse, in.uv);
    return in.color * tex_color;
}

// ============================================================================
// 2D Fragment Shader (color only, no texture)
// ============================================================================

@fragment
fn fs_main_2d_color(in: VertexOutput2D) -> @location(0) vec4<f32> {
    return in.color;
}

// ============================================================================
// 3D Vertex Shader
// ============================================================================

struct VertexInput3D {
    @location(0) position: vec3<f32>,
    @location(1) normal: vec3<f32>,
    @location(2) uv: vec2<f32>,
    @location(3) color: vec4<f32>,
}

struct VertexOutput3D {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) world_pos: vec3<f32>,
    @location(1) normal: vec3<f32>,
    @location(2) uv: vec2<f32>,
    @location(3) color: vec4<f32>,
}

@vertex
fn vs_main_3d(in: VertexInput3D) -> VertexOutput3D {
    var out: VertexOutput3D;
    out.clip_position = camera.view_proj * vec4<f32>(in.position, 1.0);
    out.world_pos = in.position;
    out.normal = in.normal;
    out.uv = in.uv;
    out.color = in.color;
    return out;
}

// ============================================================================
// 3D Fragment Shader (basic lighting)
// ============================================================================

@fragment
fn fs_main_3d(in: VertexOutput3D) -> @location(0) vec4<f32> {
    // Simple directional light
    let light_dir = normalize(vec3<f32>(0.5, 1.0, 0.3));
    let ambient = 0.2;
    let diffuse = max(dot(in.normal, light_dir), 0.0);
    let lighting = ambient + diffuse * 0.8;
    
    return vec4<f32>(in.color.rgb * lighting, in.color.a);
}
