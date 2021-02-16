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

        TextLayout::new(Point2::new(10.0, 10.0), "HELLO WORLD!", &ui.assets.medium_font)
        .render(ui_batch, ui, node, geometry, states);
    }
}

struct TextLayout{
    offset: Point2<f32>,
    glyphs: Vec<(Point2<f32>, UiTextureRegion)>,
    color: Color,
    height: f32, 
    width: f32,
}

impl TextLayout {

    pub fn new(pt: Point2<f32>, txt: &str, font: &FontMap) -> Self {
        let mut glyphs = Vec::new();

        let mut width = 0.0;

        for (index, c) in txt.chars().enumerate() {
            if c != ' '{
                let glyph = (Point2::new(width, 0.0), font.char(c));
                glyphs.push(glyph);
            }
            width += font.glyph_width; //TODO - Make fonts monospace!!
        }

        Self {
            offset: pt,
            width: glyphs.len() as f32 * font.glyph_width,
            height: font.glyph_height,
            color: Color::WHITE,
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
        let pos = Vector4::new(
            geometry.pos.x + self.offset.x,
            geometry.pos.y + self.offset.y,
            geometry.size.x,
            geometry.size.y,
        );

        for (glyph_offset, glyph) in &self.glyphs {
            ui_batch.draw(
                glyph.texture_id,
                &GPUSprite {
                    pos: Vector4::new(
                        pos.x + glyph_offset.x,
                        pos.y + glyph_offset.y,
                        pos.z,
                        pos.w,
                    ),
                    uvs: Vector4::new(
                        glyph.pos.x,
                        glyph.pos.y,
                        glyph.size.x,
                        glyph.size.y,
                    ),
                    color
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
            size: Point2::new(40.0, 80.0),
        },
        Box::new(ButtonRenderer),
        Box::new(ButtonHandler),
        Some(Box::new(ButtonState { pressed: false })),
    )
}
