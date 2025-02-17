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
    @location(0) position: vec3<f32>,
};

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) look_dir: vec3<f32>,
};

@vertex
fn vs_main(
    model: VertexInput,
) -> VertexOutput {
    var out: VertexOutput;

    out.clip_position = camera.projection * camera.view * vec4<f32>(model.position, 1.0);
    out.look_dir = model.position;

    return out;
}

// Fragment shader

const DENSITY: f32 = 0.005;
const STAR_SIZE: f32 = 0.1;

fn hash33(p: vec3<f32>) -> vec3<f32> {
    var p3: vec3<f32> = fract(p * vec3<f32>(443.897, 441.423, 437.195));
    p3 = p3 + dot(p3, p3.yxz + vec3<f32>(19.19));
    return fract((p3.xxy + p3.yxx) * p3.zyx);
}

// 3D gradient noise (simplex-like)
fn gradient_noise(p: vec3<f32>) -> f32 {
    let i = floor(p);
    let fp = fract(p);
    let u = fp * fp * (3.0 - 2.0 * fp);

    let a = hash33(i + vec3<f32>(0.0, 0.0, 0.0)).x;
    let b = hash33(i + vec3<f32>(1.0, 0.0, 0.0)).x;
    let c = hash33(i + vec3<f32>(0.0, 1.0, 0.0)).x;
    let d = hash33(i + vec3<f32>(1.0, 1.0, 0.0)).x;
    let e = hash33(i + vec3<f32>(0.0, 0.0, 1.0)).x;
    let f = hash33(i + vec3<f32>(1.0, 0.0, 1.0)).x;
    let g = hash33(i + vec3<f32>(0.0, 1.0, 1.0)).x;
    let h = hash33(i + vec3<f32>(1.0, 1.0, 1.0)).x;

    let x1 = mix(a, b, u.x);
    let x2 = mix(c, d, u.x);
    let y1 = mix(x1, x2, u.y);

    let x3 = mix(e, f, u.x);
    let x4 = mix(g, h, u.x);
    let y2 = mix(x3, x4, u.y);

    let result = mix(y1, y2, u.z);
    return result;
}

// Fractal noise (multiple octaves)
fn fractal_noise(p: vec3<f32>, octaves: i32) -> f32 {
    var value: f32 = 0.0;
    var amplitude: f32 = 0.5;
    var frequency: f32 = 1.0;

    for (var i: i32 = 0; i < octaves; i = i + 1) {
        value += amplitude * gradient_noise(p * frequency);
        amplitude *= 0.5;
        frequency *= 2.0;
    }

    return value;
}

fn calculate_stars(dir: vec3<f32>) -> f32 {
    let grid_scale = 150.0;
    let base_density = 0.9995;

    let cell = floor(dir * grid_scale);
    let rand = hash33(cell);

    if (rand.x > base_density * DENSITY) {
        return 0.0;
    }

    let star_pos = (cell + 0.5 + rand.yzx) / grid_scale;
    let dist = distance(dir, normalize(star_pos));

    let size = mix(0.001, 0.01, rand.y) * STAR_SIZE;
    let star = 1.0 - smoothstep(size * 0.8, size, dist);

    return star;
}

fn calculate_nebula(dir: vec3<f32>) -> vec3<f32> {
    let scale = 2.0;
    let animated_dir = dir;

    let noise1 = fractal_noise(animated_dir * scale, 4);
    let noise2 = fractal_noise(animated_dir * scale * 2.0, 3);
    let noise3 = fractal_noise(animated_dir * scale * 4.0, 2);

    let nebula_value = noise1 * 0.6 + noise2 * 0.3 + noise3 * 0.1;

    let intensity = smoothstep(0.2, 0.8, nebula_value*nebula_value*nebula_value) * 0.2;

    let color = mix(vec3<f32>(0.2, 0.5, 0.1), vec3<f32>(0.2, 0.1, 0.5), noise2);

    return color * intensity;
}

struct FragmentOutput {
    @location(0) color: vec4<f32>,
}

@fragment
fn fs_main(in: VertexOutput) -> FragmentOutput {
    var out: FragmentOutput;
    let dir = normalize(in.look_dir);

    var stars: f32 = calculate_stars(dir);
    stars += calculate_stars(dir*1.5) * 0.5;
    // stars += calculate_stars(dir*4.0) * 0.25;

    let nebula = calculate_nebula(dir);

    out.color = vec4<f32>(nebula + vec3<f32>(stars), 1.0);
    return out;
}
