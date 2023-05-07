#import bevy_sand::core

fn cellular_automata_rise_swap(pos: vec2<i32>)  {
	let current: Matter = read_matter(pos);
	let up: Matter = get_neighbor(pos, UP);
	let down: Matter = get_neighbor(pos, DOWN);
	var m: Matter = current;
	
    if(!is_at_border_bottom(pos) && rises_on_swap(down, current)) {
        m = down;
    } else if(!is_at_border_top(pos) && rises_on_swap(current, up)) {
        m = up;
    }
    
	write_matter(pos, m);
} 

@compute @workgroup_size(8, 8, 1)
fn main(@builtin(global_invocation_id) invocation_id: vec3<u32>)
{
	cellular_automata_rise_swap(get_current_sim_pos(invocation_id));
} 