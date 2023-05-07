#define_import_path bevy_sand::direction

const UP_LEFT: i32 = 0;
const UP: i32 = 1;
const UP_RIGHT: i32 = 2;
const RIGHT: i32 = 3;
const DOWN_RIGHT: i32 = 4;
const DOWN: i32 = 5;
const DOWN_LEFT: i32 = 6;
const LEFT: i32 = 7;

/*
Neighbor Directions
*/
const OFFSETS: array<vec2<i32>,8> = array<vec2<i32>,8>(vec2<i32>(-1, -1), vec2<i32>(0, -1), vec2<i32>(1, -1), vec2<i32>(1, 0), vec2<i32>(1, 1), vec2<i32>(0, 1), vec2<i32>(-1, 1), vec2<i32>(-1, 0));

fn get_pos_at_dir(pos: vec2<i32>, dir: i32) -> vec2<i32> {
    var offset: vec2<i32>;
    switch dir {
        case 0 {
            offset = OFFSETS[0];
        }
        case 1 {
            offset = OFFSETS[1];
        }
        case 2 {
            offset = OFFSETS[2];
        }
        case 3 {
            offset = OFFSETS[3];
        }
        case 4 {
            offset = OFFSETS[4];
        }
        case 5 {
            offset = OFFSETS[5];
        }
        case 6 {
            offset = OFFSETS[6];
        }
        case 7, default {
            offset = OFFSETS[7];
        }
    }

	return pos + offset;
} 