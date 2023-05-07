#define_import_path bevy_sand::core

#import bevy_sand::direction
#import bevy_sand::matter
#import bevy_sand::query

struct PushConstants {
    draw_start: vec2<f32>,
    draw_end: vec2<f32>,
    draw_radius: f32,
    draw_square: u32,
    sim_steps: u32,
    move_step: u32,
    dispersion_dir: u32,
    dispersion_step: u32,
    seed: f32,
    matter: Matter,
}
var<push_constant> pc: PushConstants;

@group(0) @binding(0) 
var<storage, read_write> matter_in : array<Matter>;
@group(0) @binding(1) 
var<storage, read_write> matter_out : array<Matter>;
@group(0) @binding(2)
var texture: texture_storage_2d<rgba8unorm, read_write>;

fn sim_canvas_size() -> vec2<i32> {
    return vec2<i32>(textureDimensions(texture));
}

fn get_current_sim_pos(invocation_id: vec3<u32>) -> vec2<i32> {
	return vec2<i32>(invocation_id.xy);
} 

fn get_index(location: vec2<i32>) -> i32 {
    let dims = sim_canvas_size();
    return location.y * dims.x + location.x;
}

fn read_matter(pos: vec2<i32>) -> Matter { return matter_in[get_index(pos)]; }
fn write_matter(pos: vec2<i32>, matter: Matter)  { matter_out[get_index(pos)] = matter; } 
fn write_matter_input(pos: vec2<i32>, matter: Matter)  { matter_in[get_index(pos)] = matter; }

const PHI: f32 = 1.61803398874989484820459;
fn rand(xy: vec2<i32>, seed: f32) -> f32 {
	let pos: vec2<f32> = vec2<f32>(vec2<f32>(xy).x + 0.5, vec2<f32>(xy).y + 0.5);
	return fract(tan(distance(pos * PHI, pos) * seed) * pos.x);
}