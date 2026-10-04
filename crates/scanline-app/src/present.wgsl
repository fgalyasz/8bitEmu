struct Uniforms {
    width: u32,
    height: u32,
    look: u32,
    scale: u32,
    origin_x: u32,
    origin_y: u32,
    pad0: u32,
    pad1: u32,
    palette: array<vec4<u32>, 16>,
}

@group(0) @binding(0) var<uniform> uniforms: Uniforms;
@group(0) @binding(1) var planes: texture_2d<u32>;
@group(0) @binding(2) var extra: texture_2d<u32>;

struct VertexOut {
    @builtin(position) position: vec4<f32>,
}

@vertex
fn vs_main(@builtin(vertex_index) index: u32) -> VertexOut {
    var xs = array<f32, 3>(-1.0, 3.0, -1.0);
    var ys = array<f32, 3>(-1.0, -1.0, 3.0);
    var out: VertexOut;
    out.position = vec4<f32>(xs[index], ys[index], 0.0, 1.0);
    return out;
}

@fragment
fn fs_main(@builtin(position) position: vec4<f32>) -> @location(0) vec4<f32> {
    let source = source_pixel(position);
    if source.x < 0 {
        return vec4<f32>(0.0, 0.0, 0.0, 1.0);
    }
    let color = resolve(source);
    return vec4<f32>(f32(color.r), f32(color.g), f32(color.b), 255.0) / 255.0;
}

fn source_pixel(position: vec4<f32>) -> vec2<i32> {
    let scale = max(i32(uniforms.scale), 1);
    let local_x = i32(position.x) - i32(uniforms.origin_x);
    let local_y = i32(position.y) - i32(uniforms.origin_y);
    let src = vec2<i32>(local_x / scale, local_y / scale);
    if local_x < 0 || local_y < 0 || src.x >= i32(uniforms.width) || src.y >= i32(uniforms.height) {
        return vec2<i32>(-1, -1);
    }
    return src;
}

fn resolve(source: vec2<i32>) -> vec3<u32> {
    let packed = textureLoad(planes, source, 0);
    let side = textureLoad(extra, source, 0);
    let background = temporal_color(packed.r, side.r, side.g, source);
    let covered = cover_sprite(background, packed.b, packed.a);
    return soften(covered, packed, side, source);
}

fn temporal_color(index: u32, blend: u32, previous: u32, source: vec2<i32>) -> vec3<u32> {
    let current = palette_color(index);
    if uniforms.look != 2u {
        return current;
    }
    if blend == 1u {
        return mix_half(current, palette_color(previous));
    }
    if blend == 2u {
        return mix_half(current, palette_color(neighbor_index(source)));
    }
    return current;
}

fn cover_sprite(background: vec3<u32>, sprite: u32, sprite_on: u32) -> vec3<u32> {
    if sprite_on == 0u {
        return background;
    }
    return palette_color(sprite);
}

fn soften(color: vec3<u32>, packed: vec4<u32>, side: vec4<u32>, source: vec2<i32>) -> vec3<u32> {
    if uniforms.look != 1u {
        return color;
    }
    let right = vec2<i32>(min(source.x + 1, i32(uniforms.width) - 1), source.y);
    let neighbor = textureLoad(planes, right, 0);
    let neighbor_side = textureLoad(extra, right, 0);
    if visible_luma(packed, side) == visible_luma(neighbor, neighbor_side) {
        return color;
    }
    let neighbor_color = cover_sprite(temporal_color(neighbor.r, neighbor_side.r, neighbor_side.g, right), neighbor.b, neighbor.a);
    return mix_quarter(color, neighbor_color);
}

fn visible_luma(packed: vec4<u32>, side: vec4<u32>) -> u32 {
    if packed.a != 0u {
        return side.b;
    }
    return packed.g;
}

fn neighbor_index(source: vec2<i32>) -> u32 {
    let right = vec2<i32>(min(source.x + 1, i32(uniforms.width) - 1), source.y);
    return textureLoad(planes, right, 0).r;
}

fn palette_color(index: u32) -> vec3<u32> {
    let color = uniforms.palette[index & 15u];
    return vec3<u32>(color.r, color.g, color.b);
}

fn mix_half(first: vec3<u32>, second: vec3<u32>) -> vec3<u32> {
    return mix_rgb(first, second, 1.0, 2.0);
}

fn mix_quarter(first: vec3<u32>, second: vec3<u32>) -> vec3<u32> {
    return mix_rgb(first, second, 1.0, 4.0);
}

fn mix_rgb(first: vec3<u32>, second: vec3<u32>, second_num: f32, denom: f32) -> vec3<u32> {
    return vec3<u32>(
        mix_channel(first.r, second.r, second_num, denom),
        mix_channel(first.g, second.g, second_num, denom),
        mix_channel(first.b, second.b, second_num, denom),
    );
}

fn mix_channel(first: u32, second: u32, second_num: f32, denom: f32) -> u32 {
    let mixed = (to_linear(first) * (denom - second_num) + to_linear(second) * second_num) / denom;
    return from_linear(mixed);
}

fn to_linear(channel: u32) -> f32 {
    let unit = f32(channel) / 255.0;
    if unit <= 0.04045 {
        return unit / 12.92;
    }
    return pow((unit + 0.055) / 1.055, 2.4);
}

fn from_linear(linear: f32) -> u32 {
    let clamped = clamp(linear, 0.0, 1.0);
    var encoded = 1.055 * pow(clamped, 1.0 / 2.4) - 0.055;
    if clamped <= 0.0031308 {
        encoded = 12.92 * clamped;
    }
    return u32(round(encoded * 255.0));
}
