#define_import_path bevy_shader_playground::core

struct Cell{
    alive: u32,
    heat: u32,
};

struct PushConstants {
    draw_start: vec2<f32>,
    draw_end: vec2<f32>,
    draw_radius: f32,
    draw_square: u32,
}
var<push_constant> pc: PushConstants;

fn idx(location: vec2<i32>) -> i32 {
    return location.y * i32(size.x) + location.x;
}

fn new_cell(alive: bool) -> Cell {
    return Cell(u32(alive), 0u);
}