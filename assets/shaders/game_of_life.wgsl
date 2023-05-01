@group(0) @binding(0)
var texture: texture_storage_2d<rgba16float, read_write>;

@compute @workgroup_size(32, 32, 1)
fn init(@builtin(global_invocation_id) invocation_id: vec3<u32>, @builtin(num_workgroups) num_workgroups: vec3<u32>) {}

@compute @workgroup_size(32, 32, 1)
fn update(@builtin(global_invocation_id) invocation_id: vec3<u32>) {}