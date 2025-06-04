@vertex
fn vert_main(@builtin(vertex_index) index: u32) -> @builtin(position) vec4<f32>
{
    var positions = array<vec2<f32>, 3>(
        vec2<f32>(0.0, 0.5),
        vec2<f32>(-0.5, -0.5),
        vec2<f32>(0.5, -0.5),
    );

    return vec4<f32>(positions[index], 0.0, 1.0);
}

@fragment
fn frag_main(@builtin(position) position: vec4<f32>) -> @location(0) vec4<f32> {
    return vec4<f32>(0.0, 1.0, 0.0, 1.0);
}
