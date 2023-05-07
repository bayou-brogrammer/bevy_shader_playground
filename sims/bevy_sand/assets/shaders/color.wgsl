#import bevy_sand::core

@compute @workgroup_size(8, 8, 1)
fn main(@builtin(global_invocation_id) invocation_id: vec3<u32>)
{
    let location = vec2<i32>(invocation_id.xy);
    let matter = read_matter(location);
    textureStore(texture, location, matter.color);
}