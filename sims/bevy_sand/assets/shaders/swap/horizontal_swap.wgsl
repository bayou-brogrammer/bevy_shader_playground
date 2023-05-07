#import bevy_sand::core

fn cellular_automata_move_left_swap(pos: vec2<i32>)  {
	var current: Matter = read_matter(pos);
	var right: Matter = get_neighbor(pos, RIGHT);
	var left: Matter = get_neighbor(pos, LEFT);
	let right_right: Matter = get_neighbor(get_pos_at_dir(pos, RIGHT), RIGHT);
	var m: Matter = current;

	if(!is_at_border_right(pos) && moves_on_swap_certainly(right, current, right_right)) {
        m = right;
    } else if(!is_at_border_left(pos) && moves_on_swap_certainly(current, left, right)) {
        m = left;
    } else if(!is_at_border_right(pos) && moves_on_swap_maybe(right, current, right_right, rand(get_pos_at_dir(pos, RIGHT), pc.seed))) {
        m = right;
    } else if(!is_at_border_left(pos) && moves_on_swap_maybe(current, left, right, rand(pos, pc.seed))) {
        m = left;
    }

	write_matter(pos, m);
} 

fn cellular_automata_move_right_swap(pos: vec2<i32>)  {
	let current: Matter = read_matter(pos);
	let right: Matter = get_neighbor(pos, RIGHT);
	let left: Matter = get_neighbor(pos, LEFT);
	let left_left: Matter = get_neighbor(get_pos_at_dir(pos, LEFT), LEFT);
	var m: Matter = current;

	if(!is_at_border_left(pos) && moves_on_swap_certainly(left, current, left_left)) {
        m = left;
    } else if(!is_at_border_right(pos) && moves_on_swap_certainly(current, right, left)) {
        m = right;
    } else if(!is_at_border_left(pos) && moves_on_swap_maybe(left, current, left_left, rand(get_pos_at_dir(pos, LEFT), pc.seed))) {
        m = left;
    } else if(!is_at_border_right(pos) && moves_on_swap_maybe(current, right, left, rand(pos, pc.seed))) {
        m = right;
    }

	write_matter(pos, m);
} 

fn cellular_automata_move_horizontal_swap(pos: vec2<i32>)  {
	if (pc.dispersion_dir == 0u) {
		cellular_automata_move_left_swap(pos);
	} else { 
		cellular_automata_move_right_swap(pos);
	}
} 


@compute @workgroup_size(8, 8, 1)
fn main(@builtin(global_invocation_id) invocation_id: vec3<u32>){
	cellular_automata_move_horizontal_swap(get_current_sim_pos(invocation_id));
} 
