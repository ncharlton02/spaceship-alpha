use super::*;
use cgmath::{Point2, Point3};
use std::cell::RefCell;
use winit::event;

const PADDING: f32 = 8.0;

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
        new_ninepatch_renderer(if button_state.pressed {
            ui.assets.button_pressed
        } else {
            ui.assets.button
        })
        .render(ui_batch, ui, node, geometry, states);

        let mut text = button_state.text.borrow_mut();
        text.offset.x = (geometry.size.x / 2.0) - (text.width / 2.0);
        text.render(ui_batch, ui, node, geometry, states);
    }
}

struct ButtonHandler;

impl NodeHandler for ButtonHandler {
    fn on_click(
        &self,
        _: event::MouseButton,
        click_state: event::ElementState,
        _: Point2<f32>,
        node: NodeId,
        _: &mut NodeGeometry,
        states: &mut WidgetStates,
        events: &mut EventQueue,
    ) -> bool {
        let focus = click_state == event::ElementState::Pressed;
        let mut button_state = states.get_mut::<ButtonState>(node).unwrap();
        button_state.pressed = focus;

        if focus {
            events.add(button_state.on_action.clone());
        }

        focus
    }

    fn on_mouse_focus_lost(&self, node: NodeId, states: &mut WidgetStates) {
        states.get_mut::<ButtonState>(node).unwrap().pressed = false;
    }
}

struct ButtonState {
    pressed: bool,
    text: RefCell<TextLayout>,
    on_action: EventHandler,
}

pub fn create_button(
    ui: &mut Ui,
    parent: Option<NodeId>,
    text: &str,
    on_action: EventHandler,
) -> NodeId {
    let text = TextLayout::new(
        Point2::new(PADDING, PADDING),
        text,
        &ui.assets.medium_font,
        Color::WHITE,
    );
    let min_size = Point2::new(text.width + PADDING * 2.0, text.height + PADDING * 2.0);

    ui.new_node(
        parent,
        NodeGeometry {
            pos: Point2::new(0.0, 0.0),
            size: min_size,
        },
        NodeLayout { min_size },
        Box::new(ButtonRenderer),
        Box::new(ButtonHandler),
        Some(Box::new(ButtonState {
            on_action,
            pressed: false,
            text: RefCell::new(text),
        })),
    )
}
