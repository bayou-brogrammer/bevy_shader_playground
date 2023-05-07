use bevy::{
    prelude::*,
    render::{extract_resource::ExtractResource, render_resource::Buffer, renderer::RenderDevice},
    utils::Instant,
};

use crate::constants::NUM_OF_CELLS;

#[derive(Resource, Clone, Deref, ExtractResource)]
pub struct SandPiplineImage(pub Handle<Image>);

#[repr(C)]
#[derive(Debug, Default, Clone, bytemuck::Pod, bytemuck::Zeroable, Copy)]
pub struct Matter {
    pub id: u32,
    pub weight: f32,
    pub dispersion: u32,
    _pad: u32,
    pub color: [f32; 4],
}

#[allow(dead_code)]
impl Matter {
    pub const EMPTY: Matter = Matter::new(0, [0.0, 0.0, 0.0, 1.0], 1.0, 0);
    pub const SAND: Matter = Matter::new(1, [0.76078, 0.69804, 0.50196, 1.0], 1.5, 0);
    pub const WATER: Matter = Matter::new(2, [0.01961, 0.33333, 1., 1.0], 1.0, 10);
    pub const GAS: Matter = Matter::new(3, [0.49804, 1., 0., 1.0], 0.1, 5);

    pub const fn new(id: u32, color: [f32; 4], weight: f32, dispersion: u32) -> Self {
        Self {
            id,
            color,
            weight,
            _pad: 0,
            dispersion,
        }
    }
}

#[derive(Resource)]
pub struct SandPipelineAssets {
    pub buffer_in: Buffer,
    pub buffer_out: Buffer,
}

impl FromWorld for SandPipelineAssets {
    fn from_world(w: &mut World) -> Self {
        let render_device = w.resource::<RenderDevice>();

        let initial_data = vec![Matter::EMPTY; NUM_OF_CELLS];
        let buffer_in = crate::utils::create_storage_buffer_with_data(
            render_device,
            &initial_data,
            Some("Buffer In"),
        );
        let buffer_out = crate::utils::create_storage_buffer_with_data(
            render_device,
            &initial_data,
            Some("Buffer In"),
        );

        Self {
            buffer_in,
            buffer_out,
        }
    }
}

// ================================== Constants ================================== //

#[repr(C)]
#[derive(Debug, Default, Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
pub struct SandPushConstants {
    pub draw_start: [f32; 2],
    pub draw_end: [f32; 2],
    pub draw_radius: f32,
    pub draw_square: u32,
    pub sim_step: u32,
    pub move_step: u32,
    pub dispersion_dir: u32,
    pub dispersion_step: u32,
    pub seed: f32,
    pub draw_matter: u32,
    pub matter: Matter,
}

impl SandPushConstants {
    pub fn as_bytes(&self) -> &[u8] {
        bytemuck::cast_slice(bytemuck::bytes_of(self))
    }
}

pub const INIT_MOVEMENT_STEPS: u32 = 3;
pub const INIT_DISPERSION_STEPS: u32 = 10;

#[derive(Resource, ExtractResource, Clone, Copy)]
pub struct SandAppSettings {
    pub seed: f32,
    pub start: Instant,
    pub is_paused: bool,
    pub movement_steps: u32,
    pub dispersion_steps: u32,
    pub print_performance: bool,
}

impl Default for SandAppSettings {
    fn default() -> Self {
        Self::new()
    }
}

impl SandAppSettings {
    pub fn new() -> SandAppSettings {
        let dispersion_steps = INIT_DISPERSION_STEPS;
        let movement_steps = INIT_MOVEMENT_STEPS;
        SandAppSettings {
            seed: 0.0,
            movement_steps,
            is_paused: false,
            dispersion_steps,
            start: Instant::now(),
            print_performance: false,
        }
    }

    pub fn get_current_seed(&self) -> f32 {
        (Instant::now() - self.start).as_secs_f32()
    }
}
