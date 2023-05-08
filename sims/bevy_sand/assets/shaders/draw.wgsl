#import bevy_sand::core

fn vary_color_rgb(color: vec4<f32>, seed_pos: vec2<i32>) -> vec4<f32> {
	let seed: f32 = 0.1;
	let p: f32 = rand(seed_pos, seed);
	let variation: f32 = -0.1 + 0.2 * p;
    var c = color;
	c.r = c.r + variation;
	c.g = c.g + variation;
	c.b = c.b + variation;
	return c;
}

fn variate_color(pos: vec2<i32>, color_f32: vec4<f32>) -> vec4<f32> {
	return vary_color_rgb(color_f32, pos);
}


// Line v->w, point p
// https://stackoverflow.com/questions/849211/shortest-distance-between-a-point-and-a-line-segment
fn closest_point_on_line(v: vec2<f32>, w: vec2<f32>, p: vec2<f32>) -> vec2<f32> {
    let c = v - w;
    // length squared
    let l2 = dot(c, c);
    if (l2 == 0.0) {
        return v;
    }
    let t = max(0.0, min(1.0, dot(p - v, w - v) / l2));
    let projection = v + t * (w - v);
    return projection;
}

fn color_matter_at(pos: vec2<i32>, matter: Matter){
    var m = matter;

    // 3. Vary color only if not empty
    if(!is_empty(m)) { 
        m.color = variate_color(pos, m.color); 
    }

    write_matter_input(pos, m);
}

fn draw_particle_circle(pos: vec2<f32>, draw_pos: vec2<f32>, radius: f32, matter: Matter) {
    let y_start = draw_pos.y - radius;
    let y_end = draw_pos.y + radius;
    let x_start = draw_pos.x - radius;
    let x_end = draw_pos.x + radius;
    if (pos.x >= x_start && pos.x <= x_end && pos.y >= y_start && pos.y <= y_end) {
        let diff = pos - draw_pos;
        let dist = length(diff);
        if (round(dist) <= radius) {
            color_matter_at(vec2<i32>(pos), matter);
        }
    }
}

fn draw_particle_square(pos: vec2<f32>, draw_pos: vec2<f32>, size: f32, matter: Matter)  {
	let y_start = draw_pos.y - size / 2.;
	let y_end = draw_pos.y + size / 2.;
	let x_start = draw_pos.x - size / 2.;
	let x_end = draw_pos.x + size / 2.;
	if (pos.x >= x_start && pos.x <= x_end && pos.y >= y_start && pos.y <= y_end) {
        color_matter_at(vec2<i32>(pos), matter);
	}
}

@compute @workgroup_size(8, 8, 1)
fn main(@builtin(global_invocation_id) invocation_id: vec3<u32>)
{
    let pixel = vec2<i32>(invocation_id.xy);
    let size = sim_canvas_size();
    if (pixel.x >= size.x && pixel.y >= size.y) {
        return ;
    }

    if (pc.draw_radius > 0.0) {
        let pos = vec2<f32>(pixel);
        let point_on_line = closest_point_on_line(pc.draw_start, pc.draw_end, pos);
        let matter_at = read_matter(pixel);
        let draw_matter = pc.matter;

        if(matter_at.id == empty_matter || draw_matter.id == empty_matter) {
            if (bool(pc.draw_square)){
                draw_particle_square(pos, point_on_line, pc.draw_radius,draw_matter);
            }else{
                draw_particle_circle(pos, point_on_line, pc.draw_radius,draw_matter);
            }
        }
    }
}