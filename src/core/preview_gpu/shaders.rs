#[cfg(target_os = "windows")]
// BG_DEEPEST is #09090b in sRGB. Convert to linear space for wgpu clear color.
// sRGB to linear: if c <= 0.04045 then c/12.92, else ((c+0.055)/1.055)^2.4
// #09 = 9/255 = 0.0353 -> 0.0353/12.92 = 0.00273
// #0b = 11/255 = 0.0431 -> 0.0431/12.92 = 0.00334
pub(crate) const PREVIEW_CLEAR_COLOR: wgpu::Color = wgpu::Color {
    r: 0.00273,
    g: 0.00273,
    b: 0.00334,
    a: 1.0,
};

#[cfg(target_os = "windows")]
pub(crate) const PREVIEW_SHADER: &str = r#"
struct VertexInput {
    @location(0) position: vec2<f32>,
    @location(1) uv: vec2<f32>,
};

struct VertexOutput {
    @builtin(position) position: vec4<f32>,
    @location(0) uv: vec2<f32>,
};

struct LayerUniform {
    scale_center: vec4<f32>,
    rotation_opacity: vec4<f32>,
};

@group(1) @binding(0)
var<uniform> layer: LayerUniform;

@vertex
fn vs_main(input: VertexInput) -> VertexOutput {
    var out: VertexOutput;
    let scale = layer.scale_center.xy;
    let center = layer.scale_center.zw;
    let cos_theta = layer.rotation_opacity.x;
    let sin_theta = layer.rotation_opacity.y;
    let aspect = layer.rotation_opacity.w;

    // Map 0..1 quad to centered local space, then scale.
    var local = (input.position - vec2<f32>(0.5, 0.5)) * scale;
    // Aspect-correct rotation.
    local = vec2<f32>(local.x * aspect, local.y);
    let rotated = vec2<f32>(
        local.x * cos_theta - local.y * sin_theta,
        local.x * sin_theta + local.y * cos_theta
    );
    let corrected = vec2<f32>(rotated.x / aspect, rotated.y);

    out.position = vec4<f32>(corrected + center, 0.0, 1.0);
    out.uv = input.uv;
    return out;
}

@group(0) @binding(0)
var layer_tex: texture_2d<f32>;
@group(0) @binding(1)
var layer_sampler: sampler;

@fragment
fn fs_main(input: VertexOutput) -> @location(0) vec4<f32> {
    let uv = vec2<f32>(input.uv.x, 1.0 - input.uv.y);
    var color = textureSample(layer_tex, layer_sampler, uv);
    color.a = color.a * layer.rotation_opacity.z;
    return color;
}
"#;

#[cfg(target_os = "windows")]
pub(crate) const BORDER_SHADER: &str = r#"
struct VertexInput {
    @location(0) position: vec2<f32>,
    @location(1) uv: vec2<f32>,
};

struct VertexOutput {
    @builtin(position) position: vec4<f32>,
};

struct BorderUniform {
    rect: vec4<f32>,
    color: vec4<f32>,
};

@group(0) @binding(0)
var<uniform> border: BorderUniform;

@vertex
fn vs_main(input: VertexInput) -> VertexOutput {
    var out: VertexOutput;
    let pos = border.rect.xy + input.position * border.rect.zw;
    out.position = vec4<f32>(pos, 0.0, 1.0);
    return out;
}

@fragment
fn fs_main(_input: VertexOutput) -> @location(0) vec4<f32> {
    return border.color;
}
"#;

#[cfg(target_os = "windows")]
// Border color matching PLATE_BORDER_COLOR (#27272a) in sRGB, converted to linear
// #27 = 39/255 = 0.153 sRGB -> ~0.0201 linear
// #2a = 42/255 = 0.165 sRGB -> ~0.0231 linear
pub(crate) const BORDER_COLOR_LINEAR: [f32; 4] = [0.0201, 0.0201, 0.0231, 1.0];
