mod image;
mod pipeline;

use bevy::prelude::*;
use bevy::render::extract_resource::ExtractResourcePlugin;
use bevy::render::render_graph::RenderGraph;
use bevy::{app::App, render::*};

use self::image::GameOfLifeImage;
use pipeline::{GameOfLifeNode, GameOfLifePipeline};

const SIM_SIZE: (u32, u32) = (1280, 720);
const WORKGROUP_SIZE: u32 = 8;

pub struct ShaderPlaygroundPlugin;
impl Plugin for ShaderPlaygroundPlugin {
    fn build(&self, app: &mut App) {
        app
            // Extract the game of life image resource from the main world into the render world
            // for operation on by the compute shader and display on the sprite.
            .add_plugin(ExtractResourcePlugin::<GameOfLifeImage>::default())
            .add_startup_system(setup);

        let render_app = app.sub_app_mut(RenderApp);
        render_app
            .init_resource::<GameOfLifePipeline>()
            .add_system(pipeline::queue_bind_group.in_set(RenderSet::Queue));

        let mut render_graph = render_app.world.resource_mut::<RenderGraph>();
        render_graph.add_node("game_of_life", GameOfLifeNode::default());
        render_graph.add_node_edge(
            "game_of_life",
            bevy::render::main_graph::node::CAMERA_DRIVER,
        );
    }
}

fn setup(mut commands: Commands, mut images: ResMut<Assets<Image>>) {
    let image = image::create_image(SIM_SIZE.0, SIM_SIZE.1);
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
    commands.insert_resource(image::GameOfLifeImage(image));
}
