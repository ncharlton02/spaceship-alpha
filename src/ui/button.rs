use super::*;
use cgmath::{Point2, Point3};
use winit::event;

// TODO - Use a ninepatch ui.textures.button
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
        let sprite = NodeRenderers::sprite(if button_state.pressed {
            ui.assets.button_pressed
        } else {
            ui.assets.button
        })
        .render(ui_batch, ui, node, geometry, states);

        TextLayout::new(
            Point2::new(10.0, 10.0),
            "Click Me!",
            &ui.assets.medium_font,
        )
        .render(ui_batch, ui, node, geometry, states);
    }
}

struct TextLayout {
    offset: Point2<f32>,
    glyphs: Vec<(Point2<f32>, FontGlyph)>,
    color: Color,
}

impl TextLayout {
    pub fn new(pt: Point2<f32>, txt: &str, font: &FontMap) -> Self {
        let mut glyphs = Vec::new();

        let mut width = 0.0;
        let mut last_char = None;

        for (index, c) in txt.chars().enumerate() {
            if c != ' ' {
                // TODO: Handle other whitespace
                let font_char = font.char(c);
                
                if let Some(last_char) = last_char {
                    width += font.pair_kerning(last_char, c);
                }

                glyphs.push((Point2::new(width, 0.0), font_char));
                width += font_char.advance_width;
            } else {
                width += 20.0; //TODO - add spacing to FontMap
            }

            last_char = Some(c);
        }

        Self {
            offset: pt,
            color: Color::BLACK,
            glyphs,
        }
    }
}

impl NodeRenderer for TextLayout {
    fn render(
        &self,
        ui_batch: &mut UiBatch,
        _: &Ui,
        _: NodeId,
        geometry: &NodeGeometry,
        _: &WidgetStates,
    ) {
        let color = Vector4::new(self.color.r, self.color.g, self.color.b, self.color.a);
        let pos_x = geometry.pos.x + self.offset.x;
        let pos_y = geometry.pos.y + self.offset.y;

        for (glyph_offset, glyph) in &self.glyphs {
            let texture = glyph.texture;
            ui_batch.draw(
                texture.texture_id,
                &GPUSprite {
                    pos: Vector4::new(
                        pos_x + glyph_offset.x,
                        pos_y + glyph_offset.y,
                        glyph.width,
                        glyph.height,
                    ),
                    uvs: Vector4::new(texture.pos.x, texture.pos.y, texture.size.x, texture.size.y),
                    color,
                },
            );
        }
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
}

pub fn create_button(ui: &mut Ui, parent: Option<NodeId>) -> NodeId {
    ui.new_node(
        parent,
        NodeGeometry {
            pos: Point2::new(0.0, 0.0),
            size: Point2::new(180.0, 80.0),
        },
        Box::new(ButtonRenderer),
        Box::new(ButtonHandler),
        Some(Box::new(ButtonState { pressed: false })),
    )
}
