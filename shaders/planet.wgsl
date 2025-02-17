struct CameraUniform {
    view: mat4x4<f32>,
    projection: mat4x4<f32>,
    packed_position: vec4<u32>,
    z_near: f32,
    z_far: f32,
};
@group(0) @binding(0)
var<uniform> camera: CameraUniform;

struct VertexInput {
    @location(0) position: vec4<u32>,
    @location(1) color: vec4<f32>,
};

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) color: vec3<f32>,
};

fn extract_int(position: vec4<u32>) -> vec3<i32> {
    let x_int = bitcast<i32>(position.w) >> 3; // 29 bit signed integer
    let y_int = bitcast<i32>((((position.z & ((1 << 21) - 1)) << 8) | (position.y >> 24)) << 3) >> 3; // 29 bit signed integer
    let z_int = bitcast<i32>((((position.y & ((1 << 10) - 1)) << 18) | (position.x >> 14)) << 4) >> 4; // 28 bit signed integer
    return vec3<i32>(x_int, y_int, z_int);
}

fn extract_dec(position: vec4<u32>) -> vec3<f32> {
    let x_dec = f32(((position.w & ((1 << 3) - 1)) << 11) | (position.z >> 21)); // 14 bit unsigned integer as float
    let y_dec = f32((position.y >> 10) & ((1 << 14) - 1)); // 14 bit unsigned integer as float
    let z_dec = f32(position.x & ((1 << 14) - 1)); // 14 bit unsigned integer as float
    return vec3<f32>(x_dec, y_dec, z_dec);
}

fn unpack_position(position: vec4<u32>) -> vec3<f32> {
    const SCALE: f32 = 1.0 / 16384.0;

    let cam_int = extract_int(camera.packed_position);
    let pos_int = extract_int(position);
    let cam_dec = extract_dec(camera.packed_position);
    let pos_dec = extract_dec(position);

    let rel_x = f32(pos_int.x - cam_int.x) + (pos_dec.x - cam_dec.x) * SCALE;
    let rel_y = f32(pos_int.y - cam_int.y) + (pos_dec.y - cam_dec.y) * SCALE;
    let rel_z = f32(pos_int.z - cam_int.z) + (pos_dec.z - cam_dec.z) * SCALE;

    return vec3<f32>(rel_x, rel_y, rel_z);
}

@vertex
fn vs_main(
    model: VertexInput,
) -> VertexOutput {
    var out: VertexOutput;
    out.color = model.color.xyz;

    let view_pos = camera.view * (vec4<f32>(unpack_position(model.position), 1.0));
    let z_view = -view_pos.z;
    let log_depth = (log(z_view) - log(camera.z_near)) / (log(camera.z_far) - log(camera.z_near));

    out.clip_position = camera.projection * view_pos;
    out.clip_position.z = log_depth * out.clip_position.w;

    // out.clip_position.z = log_depth * out.clip_position.w;

    return out;
}

// Fragment shader

struct FragmentOutput {
    @location(0) color: vec4<f32>,
}

@fragment
fn fs_main(in: VertexOutput) -> FragmentOutput {
    var out: FragmentOutput;
    out.color = vec4<f32>(in.color, 1.0);
    return out;
}
