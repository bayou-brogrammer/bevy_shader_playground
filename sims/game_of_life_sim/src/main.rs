// disable console on windows for release builds
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use bevy::prelude::*;
use bevy::DefaultPlugins;
use game_of_life_sim::ShaderPlaygroundPlugin;

fn main() {
    App::new()
        .insert_resource(ClearColor(Color::BLACK))
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                canvas: Some("#shader_playground".to_owned()),
                title: "Shader Playground".to_string(),
                present_mode: bevy::window::PresentMode::AutoNoVsync, // unthrottled FPS
                ..default()
            }),
            ..default()
        }))
        .add_plugin(ShaderPlaygroundPlugin)
        .run();
}
