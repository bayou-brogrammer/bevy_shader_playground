#import bevy_shader_playground::core

@group(0) @binding(0) 
var<uniform> size : vec2<u32>; // width, height
@group(0) @binding(1) 
var<storage, read_write> aliveDts : array<Cell>;
@group(0) @binding(2)
var texture: texture_storage_2d<rgba8unorm, read_write>;

fn get_cell(location: vec2<i32>) -> Cell {
    return aliveDts[idx(location)];
}

@compute @workgroup_size(8, 8, 1)
fn color(@builtin(global_invocation_id) invocation_id: vec3<u32>)
{
    let location = vec2<i32>(invocation_id.xy);
    let cell = get_cell(location);
    let alive = bool(cell.alive);

    // Alive color
    var color: vec4<f32> = vec4<f32>(f32(alive), 0., 0., 1.);

    // Dead color
    if (!alive) {
        color = vec4<f32>(0., 0., 0., 1.);

        if (cell.heat > 0u){
            color = vec4<f32>(0., 0., f32(cell.heat) / 255., 1.0);
        }
    }

    textureStore(texture, location, color);
}