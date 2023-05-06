mod camera;
mod input;
mod pipeline;
mod ui;
mod utils;

use bevy::prelude::*;
use bevy::render::extract_resource::ExtractResourcePlugin;
use bevy::{app::App, render::renderer::RenderDevice};
use input::AutomataParams;
use pipeline::automata::{GameOfLifeBuffers, GameOfLifeImage};

const WORKGROUP_SIZE: u32 = 8;
const SIM_SIZE: (u32, u32) = (1280, 720);
const NUM_OF_CELLS: usize = (SIM_SIZE.0 * SIM_SIZE.1) as usize;

pub struct ShaderPlaygroundPlugin;
impl Plugin for ShaderPlaygroundPlugin {
    fn build(&self, app: &mut App) {
        app
            // Extract the game of life image resource from the main world into the render world
            // for operation on by the compute shader and display on the sprite.
            .add_plugin(ExtractResourcePlugin::<GameOfLifeImage>::default())
            .add_plugin(ExtractResourcePlugin::<GameOfLifeBuffers>::default())
            .add_plugin(ExtractResourcePlugin::<AutomataParams>::default())
            .add_plugin(camera::CameraPlugin)
            .add_plugin(input::InputPlugin)
            .add_plugin(pipeline::PipelinesPlugin)
            .add_plugin(ui::UIPlugin)
            .add_startup_system(setup);
    }
}

fn setup(mut commands: Commands, mut images: ResMut<Assets<Image>>, device: Res<RenderDevice>) {
    let image = utils::create_image(SIM_SIZE.0, SIM_SIZE.1);
    let image = images.add(image);

    commands.spawn(SpriteBundle {
        sprite: Sprite {
            custom_size: Some(Vec2::new(SIM_SIZE.0 as f32, SIM_SIZE.1 as f32)),
            ..default()
        },
        texture: image.clone(),
        ..default()
    });

    // We multiply by 2 because we need to store `alive` and `heat` data for each cell.
    let initial_life_data = vec![0u32; 2 * NUM_OF_CELLS];
    let buffers = (0..2)
        .map(|i| {
            utils::create_storage_buffer_with_data(
                &device,
                &initial_life_data,
                Some(&format!("Game of Life Buffer {i}")),
            )
        })
        .collect::<Vec<_>>();

    let uniform_size_buffer = utils::create_uniform_buffer(
        &device,
        &[SIM_SIZE.0, SIM_SIZE.1],
        Some("Simulation Size Uniform"),
    );

    commands.spawn(Camera2dBundle::default());
    commands.insert_resource(GameOfLifeImage(image));
    commands.insert_resource(GameOfLifeBuffers {
        in_out_buffers: buffers,
        uniform_buffer: uniform_size_buffer,
    });
}
