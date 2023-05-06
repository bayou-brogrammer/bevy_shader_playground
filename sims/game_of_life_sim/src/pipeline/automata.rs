use bevy::{
    prelude::*,
    render::{
        extract_resource::ExtractResource, render_graph, render_resource::*, renderer::*, RenderSet,
    },
};
use std::borrow::Cow;

use crate::{input::AutomataParams, NUM_OF_CELLS, SIM_SIZE, WORKGROUP_SIZE};

#[derive(Resource, Clone, Deref, ExtractResource)]
pub struct GameOfLifeImage(pub Handle<Image>);

#[derive(Resource, Clone, ExtractResource)]
pub struct GameOfLifeBuffers {
    pub uniform_buffer: Buffer,
    pub in_out_buffers: Vec<Buffer>,
}

pub struct AutomataPipelinePlugin;
impl Plugin for AutomataPipelinePlugin {
    fn build(&self, render_app: &mut App) {
        render_app
            .init_resource::<AutomataPipeline>()
            .add_system(queue_automata_bind_group.in_set(RenderSet::Queue));
    }
}

// ================================== Pipeline ================================== //

#[derive(Resource)]
pub struct AutomataPipeline {
    init_pipeline: CachedComputePipelineId,
    update_pipeline: CachedComputePipelineId,
    texture_bind_group_layout: BindGroupLayout,
}

impl FromWorld for AutomataPipeline {
    fn from_world(world: &mut World) -> Self {
        let texture_bind_group_layout =
            world
                .resource::<RenderDevice>()
                .create_bind_group_layout(&BindGroupLayoutDescriptor {
                    label: Some("Game of Life Bind Group Layout"),
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
                    ],
                });

        let pipeline_cache = world.resource::<PipelineCache>();

        let main_shader = world
            .resource::<AssetServer>()
            .load("shaders/game_of_life.wgsl");

        let init_pipeline = pipeline_cache.queue_compute_pipeline(ComputePipelineDescriptor {
            layout: vec![texture_bind_group_layout.clone()],
            shader_defs: vec![],
            shader: main_shader.clone(),
            entry_point: Cow::from("init"),
            push_constant_ranges: Vec::new(),
            label: Some(std::borrow::Cow::Borrowed("Game of Life Init Pipeline")),
        });
        let update_pipeline = pipeline_cache.queue_compute_pipeline(ComputePipelineDescriptor {
            shader: main_shader,
            shader_defs: vec![],
            entry_point: Cow::from("update"),
            push_constant_ranges: Vec::new(),
            layout: vec![texture_bind_group_layout.clone()],
            label: Some(std::borrow::Cow::Borrowed("Game of Life Update Pipeline")),
        });

        AutomataPipeline {
            init_pipeline,
            update_pipeline,
            texture_bind_group_layout,
        }
    }
}

// ================================== BindGroup ================================== //

#[derive(Resource)]
pub struct AutomataTextureBindGroup(pub BindGroup);

pub fn queue_automata_bind_group(
    mut commands: Commands,
    render_device: Res<RenderDevice>,

    params: Res<AutomataParams>,
    buffers: Res<GameOfLifeBuffers>,
    pipeline: Res<AutomataPipeline>,
) {
    let (buffer_src, buffer_dst) = if *params.frame.lock() % 2 == 0 {
        (&buffers.in_out_buffers[0], &buffers.in_out_buffers[1])
    } else {
        (&buffers.in_out_buffers[1], &buffers.in_out_buffers[0])
    };

    let bind_group = render_device.create_bind_group(&BindGroupDescriptor {
        label: Some("Game of Life Bind Group"),
        layout: &pipeline.texture_bind_group_layout,
        entries: &[
            BindGroupEntry {
                binding: 0,
                resource: buffers.uniform_buffer.as_entire_binding(),
            },
            BindGroupEntry {
                binding: 1,
                resource: buffer_src.as_entire_binding(),
            },
            BindGroupEntry {
                binding: 2,
                resource: buffer_dst.as_entire_binding(),
            },
        ],
    });
    commands.insert_resource(AutomataTextureBindGroup(bind_group));
}

// ================================== Nodes ================================== //
pub enum AutomataState {
    Loading,
    Init,
    Update,
}

pub struct AutomataNode {
    state: AutomataState,
}

impl Default for AutomataNode {
    fn default() -> Self {
        Self {
            state: AutomataState::Loading,
        }
    }
}

impl render_graph::Node for AutomataNode {
    fn update(&mut self, world: &mut World) {
        let pipeline = world.resource::<AutomataPipeline>();
        let pipeline_cache = world.resource::<PipelineCache>();

        // if the corresponding pipeline has loaded, transition to the next stage
        match self.state {
            AutomataState::Loading => {
                if let CachedPipelineState::Ok(_) =
                    pipeline_cache.get_compute_pipeline_state(pipeline.init_pipeline)
                {
                    self.state = AutomataState::Init;
                }
            }
            AutomataState::Init => {
                if let CachedPipelineState::Ok(_) =
                    pipeline_cache.get_compute_pipeline_state(pipeline.update_pipeline)
                {
                    self.state = AutomataState::Update;
                }
            }
            AutomataState::Update => {
                let params = world.resource_mut::<AutomataParams>();
                if !params.is_paused {
                    *params.frame.lock() += 1;
                }
            }
        }
    }

    fn run(
        &self,
        _graph: &mut render_graph::RenderGraphContext,
        render_context: &mut RenderContext,
        world: &World,
    ) -> Result<(), render_graph::NodeRunError> {
        let is_paused = &world.resource::<AutomataParams>().is_paused;

        if !is_paused {
            let automata_bind_group = &world.resource::<AutomataTextureBindGroup>().0;
            let pipeline_cache = world.resource::<PipelineCache>();
            let pipeline = world.resource::<AutomataPipeline>();

            let mut pass = render_context
                .command_encoder()
                .begin_compute_pass(&ComputePassDescriptor::default());

            pass.set_bind_group(0, automata_bind_group, &[]);

            // select the pipeline based on the current state
            match self.state {
                AutomataState::Loading => {}
                AutomataState::Init => {
                    let init_pipeline = pipeline_cache
                        .get_compute_pipeline(pipeline.init_pipeline)
                        .unwrap();
                    pass.set_pipeline(init_pipeline);
                    pass.dispatch_workgroups(
                        SIM_SIZE.0 / WORKGROUP_SIZE,
                        SIM_SIZE.1 / WORKGROUP_SIZE,
                        1,
                    );
                }
                AutomataState::Update => {
                    let update_pipeline = pipeline_cache
                        .get_compute_pipeline(pipeline.update_pipeline)
                        .unwrap();

                    pass.set_pipeline(update_pipeline);
                    pass.dispatch_workgroups(
                        SIM_SIZE.0 / WORKGROUP_SIZE,
                        SIM_SIZE.1 / WORKGROUP_SIZE,
                        1,
                    );
                }
            }
        }

        Ok(())
    }
}
