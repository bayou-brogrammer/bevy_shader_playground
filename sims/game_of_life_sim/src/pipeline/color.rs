use bevy::{
    prelude::*,
    render::{
        render_asset::RenderAssets, render_graph, render_resource::*, renderer::*, RenderSet,
    },
};
use std::borrow::Cow;

use crate::{input::AutomataParams, NUM_OF_CELLS, SIM_SIZE, WORKGROUP_SIZE};

use super::automata::{AutomataTextureBindGroup, GameOfLifeBuffers, GameOfLifeImage};

pub struct AutomataColorPipelinePlugin;
impl Plugin for AutomataColorPipelinePlugin {
    fn build(&self, render_app: &mut App) {
        render_app
            .init_resource::<AutomataColorPipeline>()
            .add_system(queue_color_bind_group.in_set(RenderSet::Queue));
    }
}

// ================================== Pipeline ================================== //

#[derive(Resource)]
pub struct AutomataColorPipeline {
    color_pipeline: CachedComputePipelineId,
    color_bind_group_layout: BindGroupLayout,
}

impl FromWorld for AutomataColorPipeline {
    fn from_world(world: &mut World) -> Self {
        let pipeline_cache = world.resource::<PipelineCache>();

        let color_bind_group_layout =
            world
                .resource::<RenderDevice>()
                .create_bind_group_layout(&BindGroupLayoutDescriptor {
                    label: Some("Game of Life Color Bind Group Layout"),
                    entries: &[
                        BindGroupLayoutEntry {
                            binding: 0,
                            count: None,
                            visibility: ShaderStages::COMPUTE,
                            ty: BindingType::Buffer {
                                ty: BufferBindingType::Uniform,
                                has_dynamic_offset: false,
                                min_binding_size: BufferSize::new(
                                    (2 * std::mem::size_of::<u32>()) as _,
                                ),
                            },
                        },
                        BindGroupLayoutEntry {
                            binding: 1,
                            count: None,
                            visibility: ShaderStages::COMPUTE,
                            ty: BindingType::Buffer {
                                ty: BufferBindingType::Storage { read_only: false },
                                has_dynamic_offset: false,
                                min_binding_size: BufferSize::new(
                                    (2 * NUM_OF_CELLS * std::mem::size_of::<u32>()) as _,
                                ),
                            },
                        },
                        BindGroupLayoutEntry {
                            binding: 2,
                            visibility: ShaderStages::COMPUTE,
                            ty: BindingType::StorageTexture {
                                access: StorageTextureAccess::ReadWrite,
                                format: TextureFormat::Rgba8Unorm,
                                view_dimension: TextureViewDimension::D2,
                            },
                            count: None,
                        },
                    ],
                });

        let color_shader = world.resource::<AssetServer>().load("shaders/color.wgsl");

        let color_pipeline = pipeline_cache.queue_compute_pipeline(ComputePipelineDescriptor {
            shader: color_shader,
            shader_defs: vec![],
            push_constant_ranges: vec![],
            entry_point: Cow::from("color"),
            layout: vec![color_bind_group_layout.clone()],
            label: Some(std::borrow::Cow::Borrowed("Game of Life Color Pipeline")),
        });

        AutomataColorPipeline {
            color_pipeline,
            color_bind_group_layout,
        }
    }
}

// ================================== BindGroup ================================== //

#[derive(Resource)]
struct AutomataColorBindGroup(pub BindGroup);

pub fn queue_color_bind_group(
    mut commands: Commands,
    render_device: Res<RenderDevice>,
    buffers: Res<GameOfLifeBuffers>,
    pipeline: Res<AutomataColorPipeline>,
    gpu_images: Res<RenderAssets<Image>>,
    game_of_life_image: Res<GameOfLifeImage>,
    params: Res<AutomataParams>,
) {
    let view = &gpu_images[&game_of_life_image.0];
    let color_bind_group = render_device.create_bind_group(&BindGroupDescriptor {
        label: Some("Game of Life Color Bind Group"),
        layout: &pipeline.color_bind_group_layout,
        entries: &[
            BindGroupEntry {
                binding: 0,
                resource: buffers.uniform_buffer.as_entire_binding(),
            },
            BindGroupEntry {
                binding: 1,
                resource: buffers.in_out_buffers[*params.frame.lock() % 2].as_entire_binding(),
            },
            BindGroupEntry {
                binding: 2,
                resource: BindingResource::TextureView(&view.texture_view),
            },
        ],
    });
    commands.insert_resource(AutomataColorBindGroup(color_bind_group));
}

// ================================== Nodes ================================== //
pub enum AutomataColorState {
    Loading,
    Update,
}

pub struct AutomataColorNode {
    state: AutomataColorState,
}

impl Default for AutomataColorNode {
    fn default() -> Self {
        Self {
            state: AutomataColorState::Loading,
        }
    }
}

impl render_graph::Node for AutomataColorNode {
    fn update(&mut self, world: &mut World) {
        let pipeline_cache = world.resource::<PipelineCache>();
        let pipeline = world.resource::<AutomataColorPipeline>();

        // if the corresponding pipeline has loaded, transition to the next stage
        match self.state {
            AutomataColorState::Loading => {
                if let CachedPipelineState::Ok(_) =
                    pipeline_cache.get_compute_pipeline_state(pipeline.color_pipeline)
                {
                    self.state = AutomataColorState::Update;
                }
            }
            AutomataColorState::Update => {}
        }
    }

    fn run(
        &self,
        _graph: &mut render_graph::RenderGraphContext,
        render_context: &mut RenderContext,
        world: &World,
    ) -> Result<(), render_graph::NodeRunError> {
        let texture_bind_group = &world.resource::<AutomataTextureBindGroup>().0;
        let color_bind_group = &world.resource::<AutomataColorBindGroup>().0;
        let pipeline_cache = world.resource::<PipelineCache>();
        let pipeline = world.resource::<AutomataColorPipeline>();

        let mut pass = render_context
            .command_encoder()
            .begin_compute_pass(&ComputePassDescriptor::default());

        pass.set_bind_group(0, texture_bind_group, &[]);

        // select the pipeline based on the current state
        match self.state {
            AutomataColorState::Loading => {}
            AutomataColorState::Update => {
                let color_pipeline = pipeline_cache
                    .get_compute_pipeline(pipeline.color_pipeline)
                    .unwrap();

                pass.set_pipeline(color_pipeline);
                pass.set_bind_group(0, color_bind_group, &[]);
                pass.dispatch_workgroups(
                    SIM_SIZE.0 / WORKGROUP_SIZE,
                    SIM_SIZE.1 / WORKGROUP_SIZE,
                    1,
                );
            }
        }

        Ok(())
    }
}
