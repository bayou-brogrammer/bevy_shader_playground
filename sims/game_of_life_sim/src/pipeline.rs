use std::borrow::Cow;

use bevy::{
    prelude::*,
    render::{render_resource::*, renderer::RenderDevice},
};

#[derive(Resource)]
pub struct GameOfLifePipeline {
    init_pipeline: CachedComputePipelineId,
    update_pipeline: CachedComputePipelineId,
    texture_bind_group_layout: BindGroupLayout,
}

impl FromWorld for GameOfLifePipeline {
    fn from_world(world: &mut World) -> Self {
        let texture_bind_group_layout =
            world
                .resource::<RenderDevice>()
                .create_bind_group_layout(&BindGroupLayoutDescriptor {
                    label: Some("Game of Life Bind Group Layout"),
                    entries: &[BindGroupLayoutEntry {
                        binding: 0,
                        visibility: ShaderStages::COMPUTE,
                        ty: BindingType::StorageTexture {
                            access: StorageTextureAccess::ReadWrite,
                            format: TextureFormat::Rgba8Unorm,
                            view_dimension: TextureViewDimension::D2,
                        },
                        count: None,
                    }],
                });

        let pipeline_cache = world.resource::<PipelineCache>();
        let shader = world
            .resource::<AssetServer>()
            .load("shaders/game_of_life.wgsl");

        let init_pipeline = pipeline_cache.queue_compute_pipeline(ComputePipelineDescriptor {
            layout: vec![texture_bind_group_layout.clone()],
            shader_defs: vec![],
            shader: shader.clone(),
            entry_point: Cow::from("init"),
            push_constant_ranges: Vec::new(),
            label: Some(std::borrow::Cow::Borrowed("Game of Life Init Pipeline")),
        });
        let update_pipeline = pipeline_cache.queue_compute_pipeline(ComputePipelineDescriptor {
            shader,
            shader_defs: vec![],
            entry_point: Cow::from("update"),
            push_constant_ranges: Vec::new(),
            layout: vec![texture_bind_group_layout.clone()],
            label: Some(std::borrow::Cow::Borrowed("Game of Life Update Pipeline")),
        });

        GameOfLifePipeline {
            texture_bind_group_layout,
            init_pipeline,
            update_pipeline,
        }
    }
}
