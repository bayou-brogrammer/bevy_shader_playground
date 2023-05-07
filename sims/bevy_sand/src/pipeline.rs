use bevy::log;
use bevy::render::extract_resource::ExtractResourcePlugin;
use bevy::render::render_asset::RenderAssets;
use bevy::render::render_graph::{self, RenderGraph};
use bevy::render::render_resource::FilterMode;
use bevy::render::renderer::RenderDevice;
use bevy::render::{RenderApp, RenderSet};
use bevy::{asset::load_internal_asset, prelude::*, render::render_resource::*};

use crate::constants::{
    GRID_H, GRID_W, NUM_OF_CELLS, SHADER_CORE, SHADER_DIRECTION, SHADER_MATTER, SHADER_QUERY,
    SIM_SIZE,
};
use crate::input::AutomataParams;
use crate::pipeline_assets::{
    Matter, SandAppSettings, SandPipelineAssets, SandPiplineImage, SandPushConstants,
};
use crate::utils;

// ================================== Assets ================================== //

pub const PIXELS_TARGET_FORMAT: TextureFormat = TextureFormat::Rgba8Unorm;

const PIPELINE_ENTRY: &str = "main";

pub struct PipelinesPlugin;
impl Plugin for PipelinesPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<SandAppSettings>()
            .add_plugin(ExtractResourcePlugin::<SandPiplineImage>::default())
            .add_plugin(ExtractResourcePlugin::<SandAppSettings>::default())
            .add_plugin(ExtractResourcePlugin::<AutomataParams>::default())
            .add_startup_system(setup_sand_pipeline);

        load_internal_asset!(app, SHADER_CORE, "shaders/core.wgsl", Shader::from_wgsl);
        load_internal_asset!(app, SHADER_MATTER, "shaders/matter.wgsl", Shader::from_wgsl);
        load_internal_asset!(app, SHADER_DIRECTION, "shaders/dir.wgsl", Shader::from_wgsl);
        load_internal_asset!(app, SHADER_QUERY, "shaders/query.wgsl", Shader::from_wgsl);

        let render_app = app.sub_app_mut(RenderApp);
        render_app
            .init_resource::<SandPipelines>()
            .init_resource::<SandPipelineAssets>()
            .add_system(queue_bind_groups.in_set(RenderSet::Queue));

        let mut render_graph = render_app.world.resource_mut::<RenderGraph>();
        render_graph.add_node("sand_2d", Sand2DNode::default());
        render_graph.add_node_edge("sand_2d", bevy::render::main_graph::node::CAMERA_DRIVER)
    }
}

// ================================== SETUP ================================== //

pub fn setup_sand_pipeline(mut commands: Commands, mut images: ResMut<Assets<Image>>) {
    let image = utils::create_texture_2d(SIM_SIZE, PIXELS_TARGET_FORMAT, FilterMode::Nearest);
    let image = images.add(image);

    commands.spawn(SpriteBundle {
        sprite: Sprite {
            custom_size: Some(Vec2::new(SIM_SIZE.0 as f32, SIM_SIZE.1 as f32)),
            ..default()
        },
        texture: image.clone(),
        ..default()
    });

    commands.spawn(Camera2dBundle::default());
    commands.insert_resource(SandPiplineImage(image));
}

// ================================== Pipeline ================================== //

#[derive(Resource)]
pub struct SandPipelines {
    pub pipelines_bind_group_layout: BindGroupLayout,

    pub draw_pipeline: CachedComputePipelineId,
    pub color_pipeline: CachedComputePipelineId,

    pub rise_swap_pipeline: CachedComputePipelineId,
    pub rise_empty_pipeline: CachedComputePipelineId,

    pub fall_swap_pipeline: CachedComputePipelineId,
    pub fall_empty_pipeline: CachedComputePipelineId,

    pub slide_down_swap_pipeline: CachedComputePipelineId,
    pub slide_down_empty_pipeline: CachedComputePipelineId,

    pub horizontal_empty_pipeline: CachedComputePipelineId,
    pub horizontal_swap_pipeline: CachedComputePipelineId,
}

impl SandPipelines {
    fn dispatch<'a>(
        pass: &mut ComputePass<'a>,
        pipeline: &'a ComputePipeline,
        bind_group: &'a BindGroup,
        push_constants: Option<&SandPushConstants>,
    ) {
        pass.set_pipeline(pipeline);
        pass.set_bind_group(0, bind_group, &[]);

        if let Some(push_constants) = push_constants {
            pass.set_push_constants(0, push_constants.as_bytes());
        }

        pass.dispatch_workgroups(GRID_W, GRID_H, 1);
    }

    fn move_once<'a>(
        &self,
        cache: &'a PipelineCache,
        pass: &mut ComputePass<'a>,
        bind_groups: &'a SandPipelineBindGroups,
        push_constants: &mut SandPushConstants,
        move_step: u32,
    ) {
        if let (
            Some(fall_empty_pipeline),
            Some(fall_swap_pipeline),
            Some(slide_down_empty_pipeline),
            Some(slide_down_swap_pipeline),
            Some(rise_empty_pipeline),
            Some(rise_swap_pipeline),
        ) = (
            cache.get_compute_pipeline(self.fall_empty_pipeline),
            cache.get_compute_pipeline(self.fall_swap_pipeline),
            cache.get_compute_pipeline(self.slide_down_empty_pipeline),
            cache.get_compute_pipeline(self.slide_down_swap_pipeline),
            cache.get_compute_pipeline(self.rise_empty_pipeline),
            cache.get_compute_pipeline(self.rise_swap_pipeline),
        ) {
            push_constants.move_step = move_step;

            // Fall Pipelines
            SandPipelines::dispatch(
                pass,
                fall_empty_pipeline,
                &bind_groups.bind_group_main,
                None,
            );
            SandPipelines::dispatch(pass, fall_swap_pipeline, &bind_groups.bind_group_swap, None);

            // Risers
            SandPipelines::dispatch(
                pass,
                rise_empty_pipeline,
                &bind_groups.bind_group_main,
                None,
            );
            SandPipelines::dispatch(pass, rise_swap_pipeline, &bind_groups.bind_group_swap, None);

            // Sliders
            SandPipelines::dispatch(
                pass,
                slide_down_empty_pipeline,
                &bind_groups.bind_group_main,
                Some(push_constants),
            );
            SandPipelines::dispatch(
                pass,
                slide_down_swap_pipeline,
                &bind_groups.bind_group_swap,
                Some(push_constants),
            );
        }
    }

    fn disperse<'a>(
        &self,
        cache: &'a PipelineCache,
        pass: &mut ComputePass<'a>,
        bind_groups: &'a SandPipelineBindGroups,
        push_constants: &mut SandPushConstants,
        direction: u32,
        dispersion_steps: u32,
    ) {
        if let (Some(horizontal_empty_pipeline), Some(horizontal_swap_pipeline)) = (
            cache.get_compute_pipeline(self.horizontal_empty_pipeline),
            cache.get_compute_pipeline(self.horizontal_swap_pipeline),
        ) {
            push_constants.dispersion_dir = direction;
            for dispersion_step in 0..dispersion_steps {
                push_constants.dispersion_step = dispersion_step;

                SandPipelines::dispatch(
                    pass,
                    horizontal_empty_pipeline,
                    &bind_groups.bind_group_main,
                    Some(push_constants),
                );
                SandPipelines::dispatch(
                    pass,
                    horizontal_swap_pipeline,
                    &bind_groups.bind_group_swap,
                    Some(push_constants),
                );
            }
        }
    }
}

impl FromWorld for SandPipelines {
    fn from_world(world: &mut World) -> Self {
        let render_device = world.resource::<RenderDevice>();

        let pipelines_bind_group_layout =
            render_device.create_bind_group_layout(&BindGroupLayoutDescriptor {
                label: Some("color_bind_group_layout"),
                entries: &[
                    BindGroupLayoutEntry {
                        binding: 0,
                        count: None,
                        visibility: ShaderStages::COMPUTE,
                        ty: BindingType::Buffer {
                            ty: BufferBindingType::Storage { read_only: false },
                            has_dynamic_offset: false,
                            min_binding_size: BufferSize::new(
                                (std::mem::size_of::<Matter>() * NUM_OF_CELLS) as _,
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
                                (std::mem::size_of::<Matter>() * NUM_OF_CELLS) as _,
                            ),
                        },
                    },
                    // Sand texture.
                    BindGroupLayoutEntry {
                        binding: 2,
                        visibility: ShaderStages::COMPUTE,
                        ty: BindingType::StorageTexture {
                            format: PIXELS_TARGET_FORMAT,
                            access: StorageTextureAccess::ReadWrite,
                            view_dimension: TextureViewDimension::D2,
                        },
                        count: None,
                    },
                ],
            });

        let (
            // fall
            shader_fall_empty,
            shader_fall_swap,
            // slide
            shader_slide_down_empty,
            shader_slide_down_swap,
            // horizontal
            shader_horizontal_empty,
            shader_horizontal_swap,
            // risers
            shader_rise_empty,
            shader_rise_swap,
            // misc
            shader_draw,
            shader_color,
        ) = {
            let assets_server = world.resource::<AssetServer>();
            (
                // fall
                assets_server.load("shaders/empty/fall_empty.wgsl"),
                assets_server.load("shaders/swap/fall_swap.wgsl"),
                // slide
                assets_server.load("shaders/empty/slide_down_empty.wgsl"),
                assets_server.load("shaders/swap/slide_down_swap.wgsl"),
                // horizontal
                assets_server.load("shaders/empty/horizontal_empty.wgsl"),
                assets_server.load("shaders/swap/horizontal_swap.wgsl"),
                // risers
                assets_server.load("shaders/empty/rise_empty.wgsl"),
                assets_server.load("shaders/swap/rise_swap.wgsl"),
                // misc
                assets_server.load("shaders/draw.wgsl"),
                assets_server.load("shaders/color.wgsl"),
            )
        };

        let pipeline_cache = world.resource_mut::<PipelineCache>();

        let fall_empty_pipeline =
            pipeline_cache.queue_compute_pipeline(ComputePipelineDescriptor {
                shader_defs: vec![],
                shader: shader_fall_empty,
                push_constant_ranges: vec![],
                entry_point: PIPELINE_ENTRY.into(),
                label: Some("fall_empty_pipeline".into()),
                layout: vec![pipelines_bind_group_layout.clone()],
            });

        let fall_swap_pipeline = pipeline_cache.queue_compute_pipeline(ComputePipelineDescriptor {
            shader_defs: vec![],
            shader: shader_fall_swap,
            push_constant_ranges: vec![],
            entry_point: PIPELINE_ENTRY.into(),
            label: Some("fall_swap_pipeline".into()),
            layout: vec![pipelines_bind_group_layout.clone()],
        });

        let slide_down_empty_pipeline =
            pipeline_cache.queue_compute_pipeline(ComputePipelineDescriptor {
                shader_defs: vec![],
                shader: shader_slide_down_empty,
                entry_point: PIPELINE_ENTRY.into(),
                label: Some("slide_down_empty_pipeline".into()),
                layout: vec![pipelines_bind_group_layout.clone()],
                push_constant_ranges: [PushConstantRange {
                    stages: ShaderStages::COMPUTE,
                    range: 0..std::mem::size_of::<SandPushConstants>() as u32,
                }]
                .to_vec(),
            });

        let slide_down_swap_pipeline =
            pipeline_cache.queue_compute_pipeline(ComputePipelineDescriptor {
                shader_defs: vec![],
                shader: shader_slide_down_swap,
                entry_point: PIPELINE_ENTRY.into(),
                label: Some("slide_down_swap_pipeline".into()),
                layout: vec![pipelines_bind_group_layout.clone()],
                push_constant_ranges: [PushConstantRange {
                    stages: ShaderStages::COMPUTE,
                    range: 0..std::mem::size_of::<SandPushConstants>() as u32,
                }]
                .to_vec(),
            });

        let horizontal_empty_pipeline =
            pipeline_cache.queue_compute_pipeline(ComputePipelineDescriptor {
                shader_defs: vec![],
                shader: shader_horizontal_empty,
                entry_point: PIPELINE_ENTRY.into(),
                label: Some("horizontal_empty_pipeline".into()),
                layout: vec![pipelines_bind_group_layout.clone()],
                push_constant_ranges: [PushConstantRange {
                    stages: ShaderStages::COMPUTE,
                    range: 0..std::mem::size_of::<SandPushConstants>() as u32,
                }]
                .to_vec(),
            });

        let horizontal_swap_pipeline =
            pipeline_cache.queue_compute_pipeline(ComputePipelineDescriptor {
                shader_defs: vec![],
                shader: shader_horizontal_swap,
                entry_point: PIPELINE_ENTRY.into(),
                label: Some("horizontal_swap_pipeline".into()),
                layout: vec![pipelines_bind_group_layout.clone()],
                push_constant_ranges: [PushConstantRange {
                    stages: ShaderStages::COMPUTE,
                    range: 0..std::mem::size_of::<SandPushConstants>() as u32,
                }]
                .to_vec(),
            });

        let rise_empty_pipeline =
            pipeline_cache.queue_compute_pipeline(ComputePipelineDescriptor {
                shader_defs: vec![],
                push_constant_ranges: vec![],
                shader: shader_rise_empty,
                entry_point: PIPELINE_ENTRY.into(),
                label: Some("rise_empty_pipeline".into()),
                layout: vec![pipelines_bind_group_layout.clone()],
            });

        let rise_swap_pipeline = pipeline_cache.queue_compute_pipeline(ComputePipelineDescriptor {
            shader_defs: vec![],
            push_constant_ranges: vec![],
            shader: shader_rise_swap,
            entry_point: PIPELINE_ENTRY.into(),
            label: Some("rise_swap_pipeline".into()),
            layout: vec![pipelines_bind_group_layout.clone()],
        });

        let draw_pipeline = pipeline_cache.queue_compute_pipeline(ComputePipelineDescriptor {
            shader_defs: vec![],
            shader: shader_draw,
            entry_point: PIPELINE_ENTRY.into(),
            label: Some("draw_pipeline".into()),
            layout: vec![pipelines_bind_group_layout.clone()],
            push_constant_ranges: [PushConstantRange {
                stages: ShaderStages::COMPUTE,
                range: 0..std::mem::size_of::<SandPushConstants>() as u32,
            }]
            .to_vec(),
        });

        let color_pipeline = pipeline_cache.queue_compute_pipeline(ComputePipelineDescriptor {
            shader_defs: vec![],
            shader: shader_color,
            push_constant_ranges: vec![],
            entry_point: PIPELINE_ENTRY.into(),
            label: Some("color_pipeline".into()),
            layout: vec![pipelines_bind_group_layout.clone()],
        });

        SandPipelines {
            draw_pipeline,
            color_pipeline,

            fall_swap_pipeline,
            fall_empty_pipeline,

            rise_swap_pipeline,
            rise_empty_pipeline,

            slide_down_swap_pipeline,
            horizontal_swap_pipeline,

            slide_down_empty_pipeline,
            horizontal_empty_pipeline,

            pipelines_bind_group_layout,
        }
    }
}

// ================================== Bindgroups ================================== //

#[derive(Resource)]
pub struct SandPipelineBindGroups {
    pub bind_group_main: BindGroup,
    pub bind_group_swap: BindGroup,
}

fn queue_bind_groups(
    mut commands: Commands,
    params: Res<AutomataParams>,
    pipelines: Res<SandPipelines>,
    render_device: Res<RenderDevice>,
    sand_image: Res<SandPiplineImage>,
    gpu_images: Res<RenderAssets<Image>>,
    sand_compute_assets: Res<SandPipelineAssets>,
) {
    let sand_view_image = &gpu_images[&sand_image];
    let (buffer_src, buffer_dst) = if *params.frame.lock() % 2 == 0 {
        (
            &sand_compute_assets.buffer_in,
            &sand_compute_assets.buffer_out,
        )
    } else {
        (
            &sand_compute_assets.buffer_out,
            &sand_compute_assets.buffer_in,
        )
    };

    let bind_group_main = render_device.create_bind_group(&BindGroupDescriptor {
        label: "bind_group_main".into(),
        layout: &pipelines.pipelines_bind_group_layout,
        entries: &[
            BindGroupEntry {
                binding: 0,
                resource: buffer_src.as_entire_binding(),
            },
            BindGroupEntry {
                binding: 1,
                resource: buffer_dst.as_entire_binding(),
            },
            BindGroupEntry {
                binding: 2,
                resource: BindingResource::TextureView(&sand_view_image.texture_view),
            },
        ],
    });

    let bind_group_swap = render_device.create_bind_group(&BindGroupDescriptor {
        label: "bind_group_swap".into(),
        layout: &pipelines.pipelines_bind_group_layout,
        entries: &[
            BindGroupEntry {
                binding: 0,
                resource: buffer_dst.as_entire_binding(),
            },
            BindGroupEntry {
                binding: 1,
                resource: buffer_src.as_entire_binding(),
            },
            BindGroupEntry {
                binding: 2,
                resource: BindingResource::TextureView(&sand_view_image.texture_view),
            },
        ],
    });

    commands.insert_resource(SandPipelineBindGroups {
        bind_group_main,
        bind_group_swap,
    });
}

// ================================== NODE ================================== //

#[derive(Default)]
struct Sand2DNode;

impl render_graph::Node for Sand2DNode {
    fn update(&mut self, world: &mut World) {
        let params = world.resource_mut::<AutomataParams>();
        *params.frame.lock() += 1;
    }

    fn run(
        &self,
        _: &mut render_graph::RenderGraphContext,
        render_context: &mut bevy::render::renderer::RenderContext,
        world: &World,
    ) -> Result<(), render_graph::NodeRunError> {
        if let Some(pipeline_bind_groups) = world.get_resource::<SandPipelineBindGroups>() {
            let pipeline_cache = world.resource::<PipelineCache>();
            let pipelines = world.resource::<SandPipelines>();
            let params = &world.resource::<AutomataParams>();
            let settings = &world.resource::<SandAppSettings>();

            if let (Some(draw_pipeline), Some(color_pipeline)) = (
                pipeline_cache.get_compute_pipeline(pipelines.draw_pipeline),
                pipeline_cache.get_compute_pipeline(pipelines.color_pipeline),
            ) {
                let mut pass =
                    render_context
                        .command_encoder()
                        .begin_compute_pass(&ComputePassDescriptor {
                            label: Some("sand_2d"),
                        });

                let mut pc = SandPushConstants {
                    draw_radius: params.radius,
                    sim_step: params.get_frame() as u32,
                    draw_start: params.mouse_pos.to_array(),
                    draw_end: params.prev_mouse_pos.to_array(),
                    draw_square: params.use_square_brush as u32,
                    seed: settings.get_current_seed(),
                    matter: params.selected_matter,
                    ..SandPushConstants::default()
                };

                // DRAW
                if params.is_drawing {
                    pass.set_pipeline(draw_pipeline);
                    pass.set_bind_group(0, &pipeline_bind_groups.bind_group_main, &[]);
                    pass.set_push_constants(0, pc.as_bytes());
                    pass.dispatch_workgroups(GRID_W, GRID_H, 1);
                }

                if !settings.is_paused {
                    // MOVE PIPELINES
                    pipelines.move_once(
                        pipeline_cache,
                        &mut pass,
                        pipeline_bind_groups,
                        &mut pc,
                        0,
                    );
                    pipelines.disperse(
                        pipeline_cache,
                        &mut pass,
                        pipeline_bind_groups,
                        &mut pc,
                        (params.get_frame() % 2 == 0) as u32,
                        settings.dispersion_steps,
                    );

                    let mut should_disperse = false;
                    if settings.movement_steps > 1 {
                        pipelines.move_once(
                            pipeline_cache,
                            &mut pass,
                            pipeline_bind_groups,
                            &mut pc,
                            1,
                        );
                        should_disperse = true;
                    }
                    if settings.movement_steps > 2 {
                        pipelines.move_once(
                            pipeline_cache,
                            &mut pass,
                            pipeline_bind_groups,
                            &mut pc,
                            2,
                        );
                        should_disperse = true;
                    }

                    if should_disperse {
                        pipelines.disperse(
                            pipeline_cache,
                            &mut pass,
                            pipeline_bind_groups,
                            &mut pc,
                            (params.get_frame() % 2 != 0) as u32,
                            settings.dispersion_steps,
                        );
                    }
                }

                // COLOR
                SandPipelines::dispatch(
                    &mut pass,
                    color_pipeline,
                    &pipeline_bind_groups.bind_group_main,
                    None,
                );
            }
        } else {
            log::warn!("Failed to get bind groups");
        }

        Ok(())
    }
}
