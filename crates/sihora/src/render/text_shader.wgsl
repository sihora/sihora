struct Uniforms {
    viewport: vec4<f32>,
};

@group(0) @binding(0)
var<uniform> uniforms: Uniforms;

@group(1) @binding(0)
var t_atlas: texture_2d<f32>;

@group(1) @binding(1)
var s_atlas: sampler;

struct VertexInput {
    @location(0) position: vec2<f32>,
};

struct InstanceInput {
    @location(1) rect: vec4<f32>,
    @location(2) uv_rect: vec4<f32>,
    @location(3) color: vec4<f32>,
};

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) uv: vec2<f32>,
    @location(1) color: vec4<f32>,
};

@vertex
fn vs_main(vert: VertexInput, inst: InstanceInput) -> VertexOutput {
    var out: VertexOutput;
    let pos = inst.rect.xy + vert.position * inst.rect.zw;
    let ndc = pos / uniforms.viewport.xy * 2.0 - 1.0;
    out.clip_position = vec4<f32>(ndc.x, -ndc.y, 0.0, 1.0);
    out.uv = mix(inst.uv_rect.xy, inst.uv_rect.zw, vert.position);
    out.color = inst.color;
    return out;
}

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    let texel = textureSample(t_atlas, s_atlas, in.uv);
    return vec4<f32>(in.color.rgb * texel.rgb, in.color.a * texel.a);
}
