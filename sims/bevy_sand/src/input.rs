use std::sync::Arc;

use bevy::{
    input::{mouse::MouseButtonInput, ButtonState},
    prelude::*,
    render::extract_resource::{ExtractResource, ExtractResourcePlugin},
};
use bevy_egui::EguiContexts;
use parking_lot::Mutex;

use crate::{pipeline_assets::Matter, settings::SandAppSettings};

#[derive(Debug, Resource, Clone, ExtractResource)]
pub struct AutomataParams {
    pub radius: f32,
    pub mouse_pos: Vec2,
    pub can_scroll: bool,
    pub is_drawing: bool,
    pub prev_mouse_pos: Vec2,
    pub use_square_brush: bool,
    pub selected_matter: Matter,
    pub frame: Arc<Mutex<usize>>,
}

impl Default for AutomataParams {
    fn default() -> Self {
        Self {
            radius: 4.0,
            can_scroll: true,
            is_drawing: false,
            mouse_pos: Vec2::ZERO,
            use_square_brush: false,
            prev_mouse_pos: Vec2::ZERO,
            selected_matter: Matter::SAND,
            frame: Arc::new(Mutex::new(0)),
        }
    }
}

impl AutomataParams {
    pub fn get_frame(&self) -> usize {
        *self.frame.lock()
    }
}

pub struct InputPlugin;
impl Plugin for InputPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<AutomataParams>()
            .add_plugin(ExtractResourcePlugin::<AutomataParams>::default())
            .add_system(update_input_state);
    }
}

pub fn update_input_state(
    mut contexts: EguiContexts,
    window_query: Query<&Window>,
    mut params: ResMut<AutomataParams>,
    mut settings: ResMut<SandAppSettings>,
    keyboard_input: Res<Input<KeyCode>>,
    camera_q: Query<(&Camera, &GlobalTransform)>,
    mut mouse_button_input_events: EventReader<MouseButtonInput>,
) {
    let Ok(primary_window) = window_query.get_single() else { return };
    // get the camera info and transform
    let Ok((camera, camera_transform)) = camera_q.get_single() else { return };

    let ctx = contexts.ctx_mut();
    if ctx.wants_pointer_input()
        || ctx.is_pointer_over_area()
        || ctx.is_using_pointer()
        || ctx.wants_pointer_input()
    {
        // GUI gets priority input
        params.is_drawing = false;
        params.can_scroll = false;
        return;
    } else {
        params.can_scroll = true;
    }

    // Determine button state
    for event in mouse_button_input_events.iter() {
        if event.button == MouseButton::Left {
            params.is_drawing = event.state == ButtonState::Pressed;
        }
    }

    // Pause the simulation
    if keyboard_input.just_pressed(KeyCode::Space) {
        settings.is_paused = !settings.is_paused;
    }

    if let Some(world_position) = primary_window
        .cursor_position()
        .and_then(|cursor| camera.viewport_to_world(camera_transform, cursor))
        .map(|ray| ray.origin.truncate())
    {
        params.prev_mouse_pos = params.mouse_pos;
        params.mouse_pos =
            crate::utils::world_pos_to_canvas_pos(world_position * Vec2::new(1.0, -1.0));
    }
}
