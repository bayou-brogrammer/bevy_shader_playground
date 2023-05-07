#define_import_path bevy_sand::query

// Is the current thread inside the simulation canvas?
fn is_inside_sim_canvas(pos: vec2<i32>) -> bool {
	let sim_canvas_size = sim_canvas_size();
	return pos.x >= 0 && pos.x < sim_canvas_size.x && pos.y >= 0 && pos.y < sim_canvas_size.y;
} 

/*
MATTER POSITION QUERIES
*/
fn is_at_border_top(pos: vec2<i32>) -> bool { return pos.y == 0; } 
fn is_at_border_left(pos: vec2<i32>) -> bool { return pos.x == 0; } 
fn is_at_border_right(pos: vec2<i32>) -> bool { return pos.x == sim_canvas_size().x - 1; } 
fn is_at_border_bottom(pos: vec2<i32>) -> bool { return pos.y == sim_canvas_size().y - 1; } 

// | 0 1 2 |
// | 7 x 3 |
// | 6 5 4 |
fn get_neighbor(pos: vec2<i32>, dir: i32) -> Matter {
	let neighbor_pos: vec2<i32> = get_pos_at_dir(pos, dir);
	if (is_inside_sim_canvas(neighbor_pos)) {
		return read_matter(neighbor_pos);
	} else { 
		return EMPTY_MATTER;
	}
}

/*
MATTER STATE QUERIES
*/
fn is_empty(matter: Matter) -> bool { return matter.id == empty_matter; }
fn is_powder(matter: Matter) -> bool { return matter.id == state_powder; }
fn is_liquid(matter: Matter) -> bool { return matter.id == state_liquid; }
fn is_gas(matter: Matter) -> bool { return matter.id == state_gas; }
fn is_solid_gravity(matter: Matter) -> bool { return matter.id == state_solid_gravity; } 
fn is_gravity(matter: Matter) -> bool { return is_powder(matter) || is_liquid(matter) || is_solid_gravity(matter); } 

/*
MATTER MOVEMENT QUERIES
*/

/*
================== Empty ==================
*/
fn falls_on_empty(fromm: Matter, to: Matter) -> bool {
	return is_gravity(fromm) && is_empty(to);
}

fn moves_on_empty_maybe(fromm: Matter, to: Matter, opposite: Matter, down: Matter, p: f32) -> bool {
	return p < 0.5 && pc.dispersion_step < fromm.dispersion 
		&& (is_liquid(fromm) && !is_empty(down) || is_gas(fromm)) && is_empty(to) && is_empty(opposite);
} 

fn moves_on_empty_certainly(fromm: Matter, to: Matter, opposite: Matter, down: Matter) -> bool {
	return pc.dispersion_step < fromm.dispersion && (is_liquid(fromm) 
		&& !is_empty(down) || is_gas(fromm)) && is_empty(to) && !is_empty(opposite);
} 

/*
For powders
    | |f| |
    |t|x| |
    f->t where x is space under f
*/
fn slides_on_empty(from_diagonal: Matter, to_diagonal: Matter, from_down: Matter) -> bool {
	return is_powder(from_diagonal) && !is_empty(from_down) && !is_liquid(from_down) && is_empty(to_diagonal);
}

fn rises_on_empty(fromm: Matter, to: Matter) -> bool {
	return is_gas(fromm) && is_empty(to);
} 

/*
================== Swap ==================
*/

/// From could move in both direction to liquid, but takes a chance at one
/// direction
fn moves_on_swap_maybe(fromm: Matter, to: Matter, opposite: Matter, p: f32) -> bool {
	return p < 0.5 && pc.dispersion_step < fromm.dispersion 
		&& (is_liquid(fromm) || is_gas(fromm)) && (is_liquid(to) || is_gas(to)) 
		&& (is_liquid(opposite) || is_gas(opposite)) && opposite.weight < fromm.weight && to.weight < fromm.weight;
} 

fn moves_on_swap_certainly(fromm: Matter, to: Matter, opposite: Matter) -> bool {
	return pc.dispersion_step < fromm.dispersion 
		&& (is_liquid(fromm) || is_gas(fromm)) && (is_liquid(to) || is_gas(to)) 
		&& !(is_liquid(opposite) && opposite.weight < fromm.weight) && to.weight < fromm.weight;
} 

fn falls_on_swap(fromm: Matter, to: Matter) -> bool {
	return is_gravity(fromm) && (is_liquid(to) || is_gas(to)) && to.weight < fromm.weight;
}

fn slides_on_swap(from_diagonal: Matter, to_diagonal: Matter, from_down: Matter) -> bool {
	return is_powder(from_diagonal) && !is_empty(from_down) && !is_liquid(from_down) 
		&& is_liquid(to_diagonal) && to_diagonal.weight < from_diagonal.weight;
}

fn rises_on_swap(fromm: Matter, to: Matter) -> bool {
	return is_gas(fromm) && (is_liquid(to) || is_powder(to)) && to.weight > fromm.weight;
}