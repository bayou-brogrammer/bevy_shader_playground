pub mod automata;
pub mod color;
pub mod draw;

use bevy::{
    asset::load_internal_asset,
    prelude::*,
    reflect::TypeUuid,
    render::{render_graph::RenderGraph, RenderApp},
};

pub const SHADER_CORE: HandleUntyped =
    HandleUntyped::weak_from_u64(Shader::TYPE_UUID, 1371231089456109822);

pub struct PipelinesPlugin;
impl Plugin for PipelinesPlugin {
    fn build(&self, app: &mut App) {
        load_internal_asset!(app, SHADER_CORE, "shaders/core.wgsl", Shader::from_wgsl);

        let render_app = app.sub_app_mut(RenderApp);
        render_app
            .add_plugin(draw::AutomataDrawPipelinePlugin)
            .add_plugin(automata::AutomataPipelinePlugin)
            .add_plugin(color::AutomataColorPipelinePlugin);

        let mut render_graph = render_app.world.resource_mut::<RenderGraph>();
        let gol_id = render_graph.add_node("game_of_life", automata::AutomataNode::default());
        let draw_id = render_graph.add_node("game_of_life_draw", draw::AutomataDrawNode::default());
        let color_id =
            render_graph.add_node("game_of_life_color", color::AutomataColorNode::default());

        /*
         * Draw Pipeline => Automata Pipeline => Color Pipeline => Camera Driver
         */
        render_graph.add_node_edge(draw_id, gol_id);
        render_graph.add_node_edge(gol_id, color_id);
        render_graph.add_node_edge(color_id, bevy::render::main_graph::node::CAMERA_DRIVER);
    }
}
