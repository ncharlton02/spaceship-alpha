use super::*;
use cgmath::Point2;
use std::cell::RefCell;
use winit::event;

const BUTTON_PADDING: f32 = 8.0;
const LABEL_PADDING: f32 = 8.0;

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
        let button_state = states.get::<Button>(node).unwrap();
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
        let mut button_state = states.get_mut::<Button>(node).unwrap();
        button_state.pressed = focus;

        if focus {
            events.add(button_state.on_action.clone());
        }

        focus
    }

    fn on_mouse_focus_lost(&self, node: NodeId, states: &mut WidgetStates) {
        states.get_mut::<Button>(node).unwrap().pressed = false;
    }
}

pub struct Button {
    pressed: bool,
    text: RefCell<TextLayout>,
    on_action: EventHandler,
}

impl Button {
    pub fn create(
        ui: &mut Ui,
        parent: Option<NodeId>,
        text: &str,
        on_action: EventHandler,
    ) -> NodeId {
        let (text, min_size) = new_text_layout(ui, text, BUTTON_PADDING);

        ui.new_node(
            parent,
            NodeGeometry {
                pos: Point2::new(0.0, 0.0),
                size: min_size,
            },
            NodeLayout { min_size },
            Box::new(ButtonRenderer),
            Box::new(ButtonHandler),
            Some(Box::new(Button {
                on_action,
                pressed: false,
                text: RefCell::new(text),
            })),
        )
    }
}

pub fn create_texture_box(ui: &mut Ui, parent: Option<NodeId>, image: TextureRegion2D) -> NodeId {
    let min_size = image.size;
    ui.new_node(
        parent,
        NodeGeometry {
            pos: Point2::new(0.0, 0.0),
            size: min_size,
        },
        NodeLayout { min_size },
        new_sprite_renderer(image),
        Box::new(EmptyNodeHandler),
        None,
    )
}

pub struct Label {
    text: RefCell<TextLayout>,
}

impl Label {
    pub fn create(ui: &mut Ui, parent: Option<NodeId>, text: &str) -> NodeId {
        let (text, min_size) = new_text_layout(ui, text, LABEL_PADDING);

        ui.new_node(
            parent,
            NodeGeometry {
                pos: Point2::new(0.0, 0.0),
                size: min_size,
            },
            NodeLayout { min_size },
            Box::new(LabelRenderer),
            Box::new(EmptyNodeHandler),
            Some(Box::new(Label {
                text: RefCell::new(text),
            })),
        )
    }

    pub fn update_text(ui: &mut Ui, node: NodeId, text: &str) {
        let (text, min_size) = new_text_layout(ui, text, LABEL_PADDING);

        ui.layouts[node.index()].min_size = min_size;
        ui.geometries[node.arena_index()].size = min_size;
        let state = ui.states.get_mut::<Label>(node).unwrap();
        *state.text.borrow_mut() = text;
    }
}

struct LabelRenderer;

impl NodeRenderer for LabelRenderer {
    fn render(
        &self,
        ui_batch: &mut UiBatch,
        ui: &Ui,
        node: NodeId,
        geometry: &NodeGeometry,
        states: &WidgetStates,
    ) {
        let button_state = states.get::<Label>(node).unwrap();
        let mut text = button_state.text.borrow_mut();
        text.offset.x = (geometry.size.x / 2.0) - (text.width / 2.0);
        text.offset.y = (geometry.size.y / 2.0) - (text.height / 2.0);
        text.render(ui_batch, ui, node, geometry, states);
    }
}

fn new_text_layout(ui: &Ui, text: &str, padding: f32) -> (TextLayout, Point2<f32>) {
    let text = TextLayout::new(
        Point2::new(padding, padding),
        text,
        &ui.assets.medium_font,
        Color::WHITE,
    );
    let min_size = Point2::new(text.width + padding * 2.0, text.height + padding * 2.0);

    (text, min_size)
}
