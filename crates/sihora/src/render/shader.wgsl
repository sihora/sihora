struct Uniforms {
    viewport: vec4<f32>,
};

@group(0) @binding(0)
var<uniform> uniforms: Uniforms;

struct VertexInput {
    @location(0) position: vec2<f32>,
};

struct InstanceInput {
    @location(1) rect: vec4<f32>,
    @location(2) color: vec4<f32>,
    @location(3) border_color: vec4<f32>,
    @location(4) params: vec2<f32>,
};

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) local_pos: vec2<f32>,
    @location(1) size: vec2<f32>,
    @location(2) color: vec4<f32>,
    @location(3) border_color: vec4<f32>,
    @location(4) params: vec2<f32>,
};

@vertex
fn vs_main(vert: VertexInput, inst: InstanceInput) -> VertexOutput {
    var out: VertexOutput;
    let pos = inst.rect.xy + vert.position * inst.rect.zw;
    let ndc = pos / uniforms.viewport.xy * 2.0 - 1.0;
    out.clip_position = vec4<f32>(ndc.x, -ndc.y, 0.0, 1.0);
    out.local_pos = vert.position * inst.rect.zw;
    out.size = inst.rect.zw;
    out.color = inst.color;
    out.border_color = inst.border_color;
    out.params = inst.params;
    return out;
}

fn rounded_rect_sdf(p: vec2<f32>, half_ext: vec2<f32>, r: f32) -> f32 {
    let q = abs(p) - half_ext + vec2<f32>(r);
    return length(max(q, vec2<f32>(0.0))) + min(max(q.x, q.y), 0.0) - r;
}

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    let half = in.size * 0.5;
    let p = in.local_pos - half;
    let radius = min(in.params.x, min(half.x, half.y));
    let border = in.params.y;

    let d = rounded_rect_sdf(p, half, radius);
    let aa = fwidth(d);
    let outer = 1.0 - smoothstep(-0.5 * aa, 0.5 * aa, d);

    if outer < 0.001 {
        discard;
    }

    var col: vec4<f32>;
    if border > 0.0 {
        let inner_half = max(half - vec2<f32>(border), vec2<f32>(0.0));
        let inner_r = max(radius - border, 0.0);
        let id = rounded_rect_sdf(p, inner_half, inner_r);
        let inner = 1.0 - smoothstep(-0.5 * aa, 0.5 * aa, id);
        col = mix(in.border_color, in.color, inner);
    } else {
        col = in.color;
    }

    return vec4<f32>(col.rgb, col.a * outer);
}
