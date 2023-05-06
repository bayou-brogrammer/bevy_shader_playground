pub mod automata;
pub mod draw;

use bevy::{
    prelude::*,
    render::{render_graph::RenderGraph, RenderApp},
};

pub struct PipelinesPlugin;
impl Plugin for PipelinesPlugin {
    fn build(&self, app: &mut App) {
        let render_app = app.sub_app_mut(RenderApp);
        render_app
            .add_plugin(draw::AutomataDrawPipelinePlugin)
            .add_plugin(automata::AutomataPipelinePlugin);

        let mut render_graph = render_app.world.resource_mut::<RenderGraph>();
        let gol_id = render_graph.add_node("game_of_life", automata::AutomataNode::default());
        let draw_id = render_graph.add_node("game_of_life_draw", draw::AutomataDrawNode::default());

        /*
         * Draw Pipeline => Automata Pipeline => Camera Driver
         */
        render_graph.add_node_edge(draw_id, gol_id);
        render_graph.add_node_edge(gol_id, bevy::render::main_graph::node::CAMERA_DRIVER);
    }
}
