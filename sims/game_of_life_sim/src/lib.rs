mod camera;
mod input;
mod pipeline;
mod utils;

use bevy::app::App;
use bevy::prelude::*;
use bevy::render::extract_resource::ExtractResourcePlugin;
use input::InputState;
use pipeline::automata::GameOfLifeImage;

const WORKGROUP_SIZE: u32 = 8;
const SIM_SIZE: (u32, u32) = (1280, 720);

pub struct ShaderPlaygroundPlugin;
impl Plugin for ShaderPlaygroundPlugin {
    fn build(&self, app: &mut App) {
        app
            // Extract the game of life image resource from the main world into the render world
            // for operation on by the compute shader and display on the sprite.
            .add_plugin(ExtractResourcePlugin::<GameOfLifeImage>::default())
            .add_plugin(ExtractResourcePlugin::<InputState>::default())
            .add_plugin(camera::CameraPlugin)
            .add_plugin(input::InputPlugin)
            .add_plugin(pipeline::PipelinesPlugin)
            .add_startup_system(setup);
    }
}

fn setup(mut commands: Commands, mut images: ResMut<Assets<Image>>) {
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

    commands.spawn(Camera2dBundle::default());
    commands.insert_resource(GameOfLifeImage(image));
}
