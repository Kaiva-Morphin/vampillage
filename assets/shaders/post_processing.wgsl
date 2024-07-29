#import bevy_core_pipeline::fullscreen_vertex_shader::FullscreenVertexOutput

@group(0) @binding(0) var screen_texture: texture_2d<f32>;
@group(0) @binding(1) var texture_sampler: sampler;
struct PostProcessUniform {
    time: f32,
    translation: vec2<f32>,
    target_height: f32,
    target_width: f32,
    height: f32,
    width: f32,

    daytime: f32,
    day_color: vec4<f32>,
    night_color: vec4<f32>,

    vignette_strength: f32,

    wave_strength: f32,


//ifdef SIXTEEN_BYTE_ALIGNMENT
//   // WebGL2 structs must be 16 byte aligned.
//   _webgl2_padding: vec3<f32>
//endif
}
@group(0) @binding(2) var<uniform> settings: PostProcessUniform;


fn oklab_to_rgb(c: vec3<f32>) -> vec3<f32>
{
    //c.yz *= c.x;
    var l_ = c.x + 0.3963377774f * c.y + 0.2158037573f * c.z;
    var m_ = c.x - 0.1055613458f * c.y - 0.0638541728f * c.z;
    var s_ = c.x - 0.0894841775f * c.y - 1.2914855480f * c.z;

    var l = l_*l_*l_;
    var m = m_*m_*m_;
    var s = s_*s_*s_;

    var rgbResult : vec3<f32>;
    rgbResult.r =   4.0767245293*l - 3.3072168827*m + 0.2307590544*s;
    rgbResult.g = - 1.2681437731*l + 2.6093323231*m - 0.3411344290*s;
    rgbResult.b = - 0.0041119885*l - 0.7034763098*m + 1.7068625689*s;
    return rgbResult;
}

@fragment
fn fragment(in: FullscreenVertexOutput) -> @location(0) vec4<f32> {
    let time = settings.time;
    let target_aspect = settings.target_width / settings.target_height;
    let aspect = settings.width / settings.height;
    var tw = settings.target_width;
    var th = settings.target_height;

    if aspect > target_aspect {
        // keep height
        tw =  settings.target_height / settings.height * settings.width;
    } else {
        // keep width
        th =  settings.target_width / settings.width * settings.height;
    }

    let coords = in.uv * vec2(settings.width, settings.height);
    var px_coords = in.uv * vec2(tw, th);
    let px_center = vec2(tw, th) * 0.5;
    // target_aspect

    // Sample each color channel with an arbitrary shift
    //let b = vec4<f32>(
    //    textureSample(screen_texture, texture_sampler, in.uv + vec2<f32>(offset_strength, -offset_strength)).r,
    //    textureSample(screen_texture, texture_sampler, in.uv + vec2<f32>(-offset_strength, 0.0)).g,
    //    textureSample(screen_texture, texture_sampler, in.uv + vec2<f32>(0.0, offset_strength)).b,
    //    1.0
    //);
//    var c = in.position;
//    var src_col = textureSample(screen_texture, texture_sampler, in.uv);
//    src_col = src_col * 8.;
//    var col = vec4(
//        round(src_col.r),
//        round(src_col.g),
//        round(src_col.b),
//        round(src_col.a)
//    );
//    col = col / 2.;
//
//
//    return col;

    let day = vec4(1.5, 1.1, 0.6, 1.);
    let night = vec4(0.005, 0.01, 0.03, 1.);



    let daytime = settings.daytime;//(sin(time)) * 0.5 + 0.5;
    let modulate = mix(settings.day_color, settings.night_color, daytime);

    //var col = textureSample(screen_texture, texture_sampler, in.uv);

    let centered_uv = (in.uv - 0.5) * (settings.width/settings.height) * 2.0;
    let rf = sqrt(dot(centered_uv, centered_uv)) * settings.vignette_strength * (1. - daytime);
    let rf2_1 = rf * rf + 1.0;
    let vignette = 1.0 / (rf2_1 * rf2_1 * rf2_1);

    //return col * vec4(vec3(vignette), 1.0);



    //round(px.x) + round(px.y) % 2 == 0
    //if (round(px_coords.x) + round(px_coords.y)) % 2 == 0 {col = col * 0.2;}
    

    

    let pix_uv = vec2(floor(in.uv.y * (th * 0.5)) / (th * 0.5), floor(in.uv.x * (tw * 0.5)) / (tw * 0.5));
    let wave = sin(time + (pix_uv) * vec2(10., 30.)) * settings.wave_strength * 0.0001 * (1. - daytime);
    let waved_pos = in.uv + wave;
    var waved = textureSample(screen_texture, texture_sampler, waved_pos);
    if waved_pos.x < 0. || waved_pos.x > 1. {
        waved = vec4(0., 0., 0., 1.);
    }



    //let colors = 48.;
    //waved = waved * modulate * vec4(vec3(vignette), 1.0);
    //waved = waved * colors;
    //waved = vec4(floor(waved.x), floor(waved.y), floor(waved.z), colors) / colors;
    return waved * modulate * vec4(vec3(vignette), 1.0);
    //return waved;
    //return vec4(1., 1., 1., 1.) * col;
}