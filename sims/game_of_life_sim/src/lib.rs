mod image;
mod pipeline;

use bevy::diagnostic::Diagnostics;
use bevy::diagnostic::FrameTimeDiagnosticsPlugin;
use bevy::prelude::*;
use bevy::{app::App, render::*, window::PrimaryWindow};
use pipeline::GameOfLifePipeline;

const SIM_SIZE: (u32, u32) = (1280, 720);
const WORKGROUP_SIZE: u32 = 32;

pub struct ShaderPlaygroundPlugin;
impl Plugin for ShaderPlaygroundPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugin(FrameTimeDiagnosticsPlugin::default())
            .add_startup_system(setup)
            .add_system(window_fps);

        let render_app = app.sub_app_mut(RenderApp);
        render_app.init_resource::<GameOfLifePipeline>();
    }
}

fn window_fps(diagnostics: Res<Diagnostics>, mut windows: Query<&mut Window, With<PrimaryWindow>>) {
    if let Ok(mut window) = windows.get_single_mut() {
        if let Some(fps_diagnostic) = diagnostics.get(FrameTimeDiagnosticsPlugin::FPS) {
            if let Some(fps_smoothed) = fps_diagnostic.smoothed() {
                window.title = format!("{fps_smoothed:.2}");
            }
        }
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
