use bevy::{
    prelude::*,
    render::extract_resource::{ExtractResource, ExtractResourcePlugin},
    utils::Instant,
};

pub const INIT_MOVEMENT_STEPS: u32 = 3;
pub const INIT_DISPERSION_STEPS: u32 = 10;

pub struct SettingsPlugin;
impl Plugin for SettingsPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<SandAppSettings>()
            .add_plugin(ExtractResourcePlugin::<SandAppSettings>::default());
    }
}

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
