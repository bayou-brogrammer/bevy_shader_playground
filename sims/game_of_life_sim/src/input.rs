use bevy::{
    input::{mouse::MouseButtonInput, ButtonState},
    prelude::*,
    render::extract_resource::ExtractResource,
};

pub struct InputPlugin;
impl Plugin for InputPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<InputState>()
            .add_system(update_input_state);
    }
}

#[derive(Default, Resource, ExtractResource, Clone)]
pub struct InputState {
    mouse_pos: Vec2,
    prev_mouse_pos: Vec2,
    left_button_down: bool,
}

impl InputState {
    pub fn mouse_canvas_pos(&self) -> Vec2 {
        self.mouse_pos
    }

    pub fn prev_mouse_canvas_pos(&self) -> Vec2 {
        self.prev_mouse_pos
    }

    pub fn is_drawing(&self) -> bool {
        self.left_button_down
    }
}

pub fn update_input_state(
    window_query: Query<&Window>,
    mut input_state: ResMut<InputState>,
    camera_q: Query<(&Camera, &GlobalTransform)>,
    mut mouse_button_input_events: EventReader<MouseButtonInput>,
) {
    let Ok(primary_window) = window_query.get_single() else { return };
    // get the camera info and transform
    let Ok((camera, camera_transform)) = camera_q.get_single() else { return };

    // Determine button state
    for event in mouse_button_input_events.iter() {
        if event.button == MouseButton::Left {
            input_state.left_button_down = event.state == ButtonState::Pressed;
        }
    }

    if let Some(world_position) = primary_window
        .cursor_position()
        .and_then(|cursor| camera.viewport_to_world(camera_transform, cursor))
        .map(|ray| ray.origin.truncate())
    {
        input_state.prev_mouse_pos = input_state.mouse_pos;
        input_state.mouse_pos =
            crate::utils::world_pos_to_canvas_pos(world_position * Vec2::new(1.0, -1.0));
    }
}
