// disable console on windows for release builds
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use bevy::prelude::*;
use bevy::window::{close_on_esc, WindowResolution};
use bevy::DefaultPlugins;
use bevy_sand::constants::WINDOW_SIZE;

fn main() {
    App::new()
        .insert_resource(ClearColor(Color::WHITE))
        .add_plugins(
            DefaultPlugins
                .set(WindowPlugin {
                    primary_window: Some(Window {
                        resolution: WindowResolution::new(WINDOW_SIZE.0, WINDOW_SIZE.1),
                        canvas: Some("#shader_playground".to_owned()),
                        title: "Shader Playground".to_string(),
                        ..default()
                    }),
                    ..default()
                })
                .set(ImagePlugin::default_nearest()),
        )
        .add_plugin(bevy_sand::SandPlugin)
        .add_system(close_on_esc)
        .run();
}
