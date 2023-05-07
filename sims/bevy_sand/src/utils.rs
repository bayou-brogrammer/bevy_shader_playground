use bevy::prelude::Vec2;

use bevy::render::renderer::RenderDevice;
use bevy::render::texture::ImageSampler;
use bevy::{prelude::*, render::render_resource::*};

use crate::constants::SIM_SIZE;

// ================================== Render Utils ================================== //

#[rustfmt::skip]
pub fn create_texture_2d(size: (u32, u32), format: TextureFormat, filter: FilterMode) -> Image {
    let mut image = Image::new_fill(
        Extent3d {
            width: size.0,
            height: size.1,
            ..Default::default()
        },
        TextureDimension::D2,
        &[
            0, 0, 0, 0, 0, 0, 0, 0,
            0, 0, 0, 0, 0, 0, 0, 0,
            0, 0, 0, 0, 0, 0, 0, 0,
            0, 0, 0, 0, 0, 0, 0, 0,
        ],
        format,
    );

    image.texture_descriptor.usage =
        TextureUsages::COPY_DST | TextureUsages::STORAGE_BINDING | TextureUsages::TEXTURE_BINDING;

    image.sampler_descriptor = ImageSampler::Descriptor(SamplerDescriptor {
        mag_filter: filter,
        min_filter: filter,
        address_mode_u: AddressMode::ClampToBorder,
        address_mode_v: AddressMode::ClampToBorder,
        address_mode_w: AddressMode::ClampToBorder,
        ..Default::default()
    });

    image
}

pub fn create_storage_buffer_with_data<T: bytemuck::Pod + bytemuck::Zeroable>(
    device: &RenderDevice,
    data: &[T],
    label: Option<&str>,
) -> Buffer {
    device.create_buffer_with_data(&BufferInitDescriptor {
        label,
        contents: bytemuck::cast_slice(data),
        usage: BufferUsages::STORAGE | BufferUsages::COPY_DST,
    })
}

// ================================== Camera ================================== //

pub fn world_pos_to_canvas_pos(world_pos: Vec2) -> Vec2 {
    world_pos + Vec2::new(SIM_SIZE.0 as f32 / 2.0, SIM_SIZE.1 as f32 / 2.0)
}
