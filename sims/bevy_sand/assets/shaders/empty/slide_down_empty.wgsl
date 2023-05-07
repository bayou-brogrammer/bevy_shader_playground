#import bevy_sand::core

fn cellular_automata_slide_left_empty(pos: vec2<i32>)  {
	var current: Matter = read_matter(pos);
	var down: Matter = get_neighbor(pos, DOWN);
	let right: Matter = get_neighbor(pos, RIGHT);
	let up_right: Matter = get_neighbor(pos, UP_RIGHT);
	let down_left: Matter = get_neighbor(pos, DOWN_LEFT);
	var m: Matter = current;
	
	if(!is_at_border_top(pos) && !is_at_border_right(pos) && slides_on_empty(up_right, current, right)) {
		m = up_right;
	} else if(!is_at_border_bottom(pos) && !is_at_border_left(pos) && slides_on_empty(current, down_left, down)) {
		m = down_left;
	}
	write_matter(pos, m);
} 

fn cellular_automata_slide_right_empty(pos: vec2<i32>)  {
	let current: Matter = read_matter(pos);
	let down: Matter = get_neighbor(pos, DOWN);
	let left: Matter = get_neighbor(pos, LEFT);
	let up_left: Matter = get_neighbor(pos, UP_LEFT);
	let down_right: Matter = get_neighbor(pos, DOWN_RIGHT);
	var m: Matter = current;
	
	if(!is_at_border_top(pos) && !is_at_border_left(pos) && slides_on_empty(up_left, current, left)) {
		m = up_left;
	} else if(!is_at_border_bottom(pos) && !is_at_border_right(pos) && slides_on_empty(current, down_right, down)) {
		m = down_right;
	}

	write_matter(pos, m);
}

fn cellular_automata_slide_down_empty(pos: vec2<i32>)  {
	if ((pc.sim_steps + pc.move_step) % 2u == 0u) {
		cellular_automata_slide_left_empty(pos);
	} else { 
		cellular_automata_slide_right_empty(pos);
	}
} 

@compute @workgroup_size(8, 8, 1)
fn main(@builtin(global_invocation_id) invocation_id: vec3<u32>)
{
	cellular_automata_slide_down_empty(get_current_sim_pos(invocation_id));
} 