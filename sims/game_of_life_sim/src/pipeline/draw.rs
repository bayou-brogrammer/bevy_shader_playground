use bevy::{
    prelude::*,
    render::{render_graph, render_resource::*, renderer::*, RenderSet},
};
use std::borrow::Cow;

use crate::{input::AutomataParams, NUM_OF_CELLS, SIM_SIZE, WORKGROUP_SIZE};

use super::automata::{AutomataTextureBindGroup, GameOfLifeBuffers};

pub struct AutomataDrawPipelinePlugin;
impl Plugin for AutomataDrawPipelinePlugin {
    fn build(&self, render_app: &mut App) {
        render_app
            .init_resource::<AutomataDrawPipeline>()
            .add_system(queue_draw_bind_group.in_set(RenderSet::Queue));
    }
}

// ================================== Constants ================================== //

#[repr(C)]
#[derive(Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
pub struct AutomataPushConstants {
    draw_start: [f32; 2],
    draw_end: [f32; 2],
    draw_radius: f32,
    draw_square: u32,
}

impl AutomataPushConstants {
    pub fn new(draw_start: Vec2, draw_end: Vec2, draw_radius: f32, draw_square: bool) -> Self {
        Self {
            draw_radius,
            draw_end: draw_end.to_array(),
            draw_square: draw_square as u32,
            draw_start: draw_start.to_array(),
        }
    }
}

// ================================== Pipeline ================================== //

#[derive(Resource)]
pub struct AutomataDrawPipeline {
    draw_pipeline: CachedComputePipelineId,
    draw_bind_group_layout: BindGroupLayout,
}

impl FromWorld for AutomataDrawPipeline {
    fn from_world(world: &mut World) -> Self {
        let pipeline_cache = world.resource::<PipelineCache>();

        let draw_bind_group_layout =
            world
                .resource::<RenderDevice>()
                .create_bind_group_layout(&BindGroupLayoutDescriptor {
                    label: Some("Game of Life Draw Bind Group Layout"),
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
                    ],
                });

        let brush_shader = world.resource::<AssetServer>().load("shaders/draw.wgsl");

        let draw_pipeline = pipeline_cache.queue_compute_pipeline(ComputePipelineDescriptor {
            shader: brush_shader,
            shader_defs: vec![],
            entry_point: Cow::from("draw"),
            layout: vec![draw_bind_group_layout.clone()],
            label: Some(std::borrow::Cow::Borrowed("Game of Life Draw Pipeline")),
            push_constant_ranges: [PushConstantRange {
                stages: ShaderStages::COMPUTE,
                range: 0..std::mem::size_of::<AutomataPushConstants>() as u32,
            }]
            .to_vec(),
        });

        AutomataDrawPipeline {
            draw_pipeline,
            draw_bind_group_layout,
        }
    }
}

// ================================== BindGroup ================================== //

#[derive(Resource)]
struct AutomataDrawBindGroup(pub BindGroup);

pub fn queue_draw_bind_group(
    mut commands: Commands,
    render_device: Res<RenderDevice>,
    buffers: Res<GameOfLifeBuffers>,
    pipeline: Res<AutomataDrawPipeline>,
    params: Res<AutomataParams>,
) {
    let draw_bind_group = render_device.create_bind_group(&BindGroupDescriptor {
        label: Some("Game of Life Draw Bind Group"),
        layout: &pipeline.draw_bind_group_layout,
        entries: &[
            BindGroupEntry {
                binding: 0,
                resource: buffers.uniform_buffer.as_entire_binding(),
            },
            BindGroupEntry {
                binding: 1,
                resource: buffers.in_out_buffers[*params.frame.lock() % 2].as_entire_binding(),
            },
        ],
    });
    commands.insert_resource(AutomataDrawBindGroup(draw_bind_group));
}

// ================================== Nodes ================================== //
pub enum AutomataDrawState {
    Loading,
    Update,
}

pub struct AutomataDrawNode {
    state: AutomataDrawState,
}

impl Default for AutomataDrawNode {
    fn default() -> Self {
        Self {
            state: AutomataDrawState::Loading,
        }
    }
}

impl render_graph::Node for AutomataDrawNode {
    fn update(&mut self, world: &mut World) {
        let pipeline_cache = world.resource::<PipelineCache>();
        let pipeline = world.resource::<AutomataDrawPipeline>();

        // if the corresponding pipeline has loaded, transition to the next stage
        match self.state {
            AutomataDrawState::Loading => {
                if let CachedPipelineState::Ok(_) =
                    pipeline_cache.get_compute_pipeline_state(pipeline.draw_pipeline)
                {
                    self.state = AutomataDrawState::Update;
                }
            }
            AutomataDrawState::Update => {}
        }
    }

    fn run(
        &self,
        _graph: &mut render_graph::RenderGraphContext,
        render_context: &mut RenderContext,
        world: &World,
    ) -> Result<(), render_graph::NodeRunError> {
        let params = &world.resource::<AutomataParams>();

        if params.is_drawing {
            let texture_bind_group = &world.resource::<AutomataTextureBindGroup>().0;
            let draw_bind_group = &world.resource::<AutomataDrawBindGroup>().0;
            let pipeline_cache = world.resource::<PipelineCache>();
            let pipeline = world.resource::<AutomataDrawPipeline>();

            let mut pass = render_context
                .command_encoder()
                .begin_compute_pass(&ComputePassDescriptor::default());

            pass.set_bind_group(0, texture_bind_group, &[]);

            // select the pipeline based on the current state
            match self.state {
                AutomataDrawState::Loading => {}
                AutomataDrawState::Update => {
                    let draw_pipeline = pipeline_cache
                        .get_compute_pipeline(pipeline.draw_pipeline)
                        .unwrap();

                    let pc = AutomataPushConstants::new(
                        params.mouse_pos,
                        params.prev_mouse_pos,
                        params.radius,
                        params.use_square_brush,
                    );

                    pass.set_pipeline(draw_pipeline);
                    pass.set_bind_group(0, draw_bind_group, &[]);
                    pass.set_push_constants(0, bytemuck::cast_slice(&[pc]));
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
