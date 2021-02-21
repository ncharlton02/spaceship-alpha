use super::*;
use cgmath::{Point2, Point3};
use winit::event;

struct ButtonRenderer;

impl NodeRenderer for ButtonRenderer {
    fn render(
        &self,
        ui_batch: &mut UiBatch,
        ui: &Ui,
        node: NodeId,
        geometry: &NodeGeometry,
        states: &WidgetStates,
    ) {
        let button_state = states.get::<ButtonState>(node).unwrap();
        let sprite = new_sprite_renderer(if button_state.pressed {
            ui.assets.button_pressed
        } else {
            ui.assets.button
        })
        .render(ui_batch, ui, node, geometry, states);

        button_state
            .text
            .render(ui_batch, ui, node, geometry, states);
    }
}

struct ButtonHandler;

impl NodeHandler for ButtonHandler {
    fn on_click(
        &self,
        _: event::MouseButton,
        state: event::ElementState,
        _: Point2<f32>,
        node: NodeId,
        _: &mut NodeGeometry,
        states: &mut WidgetStates,
    ) -> bool {
        let focus = state == event::ElementState::Pressed;
        states.get_mut::<ButtonState>(node).unwrap().pressed = focus;

        focus
    }

    fn on_mouse_focus_lost(&self, node: NodeId, states: &mut WidgetStates) {
        states.get_mut::<ButtonState>(node).unwrap().pressed = false;
    }
}

struct ButtonState {
    pressed: bool,
    text: TextLayout,
}

pub fn create_button(ui: &mut Ui, parent: Option<NodeId>) -> NodeId {
    let padding = 10.0;
    let text = TextLayout::new(
        Point2::new(padding, padding),
        "abcdefghijklmnopqrstuvwxyz",
        &ui.assets.medium_font,
        Color::BLACK,
    );

    ui.new_node(
        parent,
        NodeGeometry {
            pos: Point2::new(0.0, 0.0),
            size: Point2::new(text.width + padding * 2.0, text.height + padding * 2.0),
        },
        Box::new(ButtonRenderer),
        Box::new(ButtonHandler),
        Some(Box::new(ButtonState {
            pressed: false,
            text,
        })),
    )
}
