#define_import_path bevy_sand::matter

const empty_matter: u32 = 0u;
const state_powder: u32 = 1u;
const state_liquid: u32 = 2u;
const state_gas: u32 = 3u;
const state_solid_gravity: u32 = 4u;

struct Matter{
    id: u32,
    weight: f32,
    dispersion: u32,
    color: vec4<f32>,
}

const EMPTY_COLOR: vec4<f32> = vec4<f32>(0.0, 0.0, 0.0, 1.0);
const SAND_COLOR: vec4<f32> = vec4<f32>(0.76078, 0.69804, 0.50196, 1.0);
const WATER_COLOR: vec4<f32> = vec4<f32>(0.01961, 0.33333, 1., 1.0);

const EMPTY_MATTER: Matter = Matter(empty_matter, 0.0, 0u, EMPTY_COLOR);
const SAND: Matter = Matter(state_powder, 1.5, 0u, SAND_COLOR);
const WATER: Matter = Matter(state_liquid, 1.0, 10u, WATER_COLOR);

fn new_matter(id: u32) -> Matter {
    var matter = EMPTY_MATTER;

    switch id {
        case 1u: {
            matter = SAND;
        }
        case 2u: {
            matter = WATER;
        }
        default {
            matter = EMPTY_MATTER;
        }
    }

    return matter;
}