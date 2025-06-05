@group(0) @binding(0)
var<uniform> uniforms: Uniforms;

struct Uniforms {
    transform: mat4x4<f32>,
}

struct VertexInput {
    @location(0) vertex_position: vec3<f32>,
    @location(1) instance_position: vec3<f32>,
}

struct VertexOutput {
    @builtin(position) position: vec4<f32>,
}

@vertex
fn vert_main(in: VertexInput) -> VertexOutput {
    let position = in.instance_position + in.vertex_position;

    var output: VertexOutput;
    output.position = uniforms.transform * vec4<f32>(position, 1.0);

    return output;
}

@fragment
fn frag_main(in: VertexOutput) -> @location(0) vec4<f32> {
    return vec4<f32>(0.0, 1.0, 0.0, 1.0);
}
