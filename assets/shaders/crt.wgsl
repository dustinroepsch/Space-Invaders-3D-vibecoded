#import bevy_core_pipeline::fullscreen_vertex_shader::FullscreenVertexOutput

@group(0) @binding(0) var screen_texture: texture_2d<f32>;
@group(0) @binding(1) var texture_sampler: sampler;

/// Barrel distortion warps the image inward, simulating a convex CRT screen.
fn barrel_distort(uv: vec2<f32>, strength: f32) -> vec2<f32> {
    let c = uv * 2.0 - 1.0;
    let r2 = dot(c, c);
    return (c * (1.0 + strength * r2)) * 0.5 + 0.5;
}

@fragment
fn fragment(in: FullscreenVertexOutput) -> @location(0) vec4<f32> {
    // ── Screen curvature ─────────────────────────────────────────────────────
    let uv = barrel_distort(in.uv, 0.08);

    // Pixels outside the distorted region become the "bezel" (pure black).
    if uv.x < 0.0 || uv.x > 1.0 || uv.y < 0.0 || uv.y > 1.0 {
        return vec4<f32>(0.0, 0.0, 0.0, 1.0);
    }

    // ── Chromatic aberration ──────────────────────────────────────────────────
    // R channel shifts slightly right, B channel shifts slightly left.
    let ca = 0.0022;
    let r = textureSample(screen_texture, texture_sampler, uv + vec2<f32>( ca, 0.0)).r;
    let g = textureSample(screen_texture, texture_sampler, uv).g;
    let b = textureSample(screen_texture, texture_sampler, uv - vec2<f32>( ca, 0.0)).b;
    var color = vec3<f32>(r, g, b);

    // ── Scanlines ─────────────────────────────────────────────────────────────
    // Every other pixel row is dimmed slightly to mimic phosphor line gaps.
    let row = floor(in.position.y);
    color *= 1.0 - 0.14 * (row % 2.0);

    // ── Vignette ──────────────────────────────────────────────────────────────
    // Darken the corners and edges like a real CRT tube.
    let p = uv * 2.0 - 1.0;
    let vig = 1.0 - dot(p * vec2<f32>(0.65, 1.0), p * vec2<f32>(0.65, 1.0)) * 0.28;
    color *= clamp(vig, 0.0, 1.0);

    // ── Subtle phosphor warm tint ─────────────────────────────────────────────
    // Very mild green-warm tint reminiscent of early CRT phosphors.
    color = mix(color, color * vec3<f32>(0.90, 1.0, 0.84), 0.07);

    return vec4<f32>(color, 1.0);
}
