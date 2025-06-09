@group(0) @binding(0)
var<uniform> uniforms: Uniforms;

struct Uniforms {
    transform: mat4x4<f32>,
}

struct VertexInput {
    @location(0) instance_position: vec3<f32>,
    @location(1) instance_color: vec4<f32>,
    @location(2) vertex_position: vec3<f32>,
}

struct VertexOutput {
    @builtin(position) position: vec4<f32>,
    @location(1) color: vec4<f32>,
}

@vertex
fn vert_main(in: VertexInput) -> VertexOutput {
    let position = in.instance_position + in.vertex_position;

    var out: VertexOutput;
    out.position = uniforms.transform * vec4<f32>(position, 1.0);
    out.color = in.instance_color;

    return out;
}

@fragment
fn frag_main(in: VertexOutput) -> @location(0) vec4<f32> {
    return in.color;
}
