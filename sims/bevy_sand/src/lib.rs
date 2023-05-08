mod camera;
pub mod constants;
mod input;
mod pipeline;
mod pipeline_assets;
mod settings;
mod ui;
mod utils;

use bevy_fn_plugin::bevy_plugin;

#[bevy_plugin]
pub fn SandPlugin(app: &mut App) {
    app.add_plugin(settings::SettingsPlugin)
        .add_plugin(input::InputPlugin)
        .add_plugin(camera::CameraPlugin)
        .add_plugin(ui::SandUIPlugin)
        .add_plugin(pipeline::PipelinesPlugin);
}
