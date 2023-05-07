use bevy::{
    prelude::{HandleUntyped, Shader},
    reflect::TypeUuid,
};

pub const WORKGROUP_SIZE: u32 = 8;
pub const GRID_W: u32 = SIM_SIZE.0 / WORKGROUP_SIZE;
pub const GRID_H: u32 = SIM_SIZE.1 / WORKGROUP_SIZE;

pub const SIM_SIZE: (u32, u32) = (720, 720);
pub const NUM_OF_CELLS: usize = (SIM_SIZE.0 * SIM_SIZE.1) as usize;

pub const SHADER_CORE: HandleUntyped =
    HandleUntyped::weak_from_u64(Shader::TYPE_UUID, 1371231089456109822);
pub const SHADER_DIRECTION: HandleUntyped =
    HandleUntyped::weak_from_u64(Shader::TYPE_UUID, 4462033275253590181);
pub const SHADER_QUERY: HandleUntyped =
    HandleUntyped::weak_from_u64(Shader::TYPE_UUID, 5254739165481917368);
pub const SHADER_MATTER: HandleUntyped =
    HandleUntyped::weak_from_u64(Shader::TYPE_UUID, 1287391288877821366);

// 2387462894328787238
// 9876835068496322894
