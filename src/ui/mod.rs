use cgmath::{Point2, Vector2, Vector4};

use crate::graphics::{
    FontGlyph, FontMap, GPUSprite, UiAssets, UiBatch, UiTextureId, UiTextureRegion,
};
use generational_arena::Arena;
use std::any::Any;
use winit::event;

pub use button::*;

mod button;
pub mod widget_textures;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct NodeId(generational_arena::Index);

impl NodeId {
    fn arena_index(&self) -> generational_arena::Index {
        self.0
    }

    pub fn index(&self) -> usize {
        self.0.into_raw_parts().0
    }
}

pub struct NodeGeometry {
    pub pos: Point2<f32>,
    pub size: Point2<f32>,
}

pub struct WidgetStates {
    states: Vec<Option<Box<dyn Any>>>,
}

impl WidgetStates {
    pub fn get<C: 'static>(&self, node: NodeId) -> Option<&C> {
        if node.index() >= self.states.len() {
            panic!("Invalid ID: {:?}", node);
        }

        if let Some(state) = &self.states[node.index()] {
            if state.is::<C>() {
                return state.downcast_ref::<C>();
            }
        }

        None
    }

    pub fn get_mut<C: 'static>(&mut self, node: NodeId) -> Option<&mut C> {
        if node.index() >= self.states.len() {
            panic!("Invalid ID: {:?}", node);
        }

        if let Some(state) = &mut self.states[node.index()] {
            if state.is::<C>() {
                return state.downcast_mut::<C>();
            }
        }

        None
    }
}

pub struct Ui {
    geometries: Arena<NodeGeometry>,
    parents: Vec<Option<NodeId>>,
    children: Vec<Vec<NodeId>>,
    handlers: Vec<Box<dyn NodeHandler>>,
    renderers: Vec<Box<dyn NodeRenderer>>,
    states: WidgetStates,
    assets: UiAssets,
    mouse_focus: Option<NodeId>,
}

impl Ui {
    pub fn new(assets: UiAssets) -> Self {
        let mut ui = Self {
            geometries: Arena::new(),
            parents: Vec::new(),
            children: Vec::new(),
            renderers: Vec::new(),
            handlers: Vec::new(),
            states: WidgetStates { states: Vec::new() },
            mouse_focus: None,
            assets,
        };

        let root = create_button(&mut ui, None);
        let _child = ui.new_node(
            Some(root),
            NodeGeometry {
                pos: Point2::new(0.0, 0.0),
                size: Point2::new(1.0, 1.0),
            },
            Box::new(EmptyRenderer),
            Box::new(EmptyNodeHandler),
            None,
        );

        ui
    }

    pub fn new_node(
        &mut self,
        parent: Option<NodeId>,
        geometry: NodeGeometry,
        renderer: Box<dyn NodeRenderer>,
        handler: Box<dyn NodeHandler>,
        state: Option<Box<dyn Any>>,
    ) -> NodeId {
        let id = NodeId(self.geometries.insert(geometry));
        insert_or_replace(&mut self.parents, id, parent);
        insert_or_replace(&mut self.children, id, Vec::new());
        insert_or_replace(&mut self.renderers, id, renderer);
        insert_or_replace(&mut self.handlers, id, handler);
        insert_or_replace(&mut self.states.states, id, state);

        if let Some(parent) = parent {
            self.check_id(parent, "Invalid parent.");
            self.children[parent.index()].push(id);
        }

        id
    }

    // Removes a node, and all of its children
    pub fn remove_node(&mut self, id: NodeId) {
        self.check_id(id, "Failed to remove node!");

        self.geometries.remove(id.arena_index());
        self.parents[id.index()] = None;
        self.renderers[id.index()] = Box::new(EmptyRenderer);
        self.handlers[id.index()] = Box::new(EmptyNodeHandler);
        self.states.states[id.index()] = None;

        std::mem::replace(&mut self.children[id.index()], Vec::with_capacity(0))
            .iter()
            .for_each(|child| self.remove_node(*child));
    }

    pub fn render(&self, sprite_batch: &mut UiBatch) {
        sprite_batch.reset();
        self.geometries.iter().for_each(|(id, geometry)| {
            let renderer = self.renderers.get(id.into_raw_parts().0).unwrap();
            renderer.render(sprite_batch, &self, NodeId(id), geometry, &self.states)
        });
    }

    pub fn on_click(
        &mut self,
        button: event::MouseButton,
        state: event::ElementState,
        pt: Point2<f32>,
    ) {
        let widgets: Vec<generational_arena::Index> = self
            .geometries
            .iter()
            .filter(|(_, geometry)| {
                pt.x > geometry.pos.x
                    && pt.y > geometry.pos.y
                    && pt.x < geometry.pos.x + geometry.size.x
                    && pt.y < geometry.pos.y + geometry.size.y
            })
            .map(|(index, _)| index)
            .collect();

        let mut new_focus = None;
        for id in widgets {
            let index = id.into_raw_parts().0;

            if let Some(handler) = self.handlers.get(index) {
                let node_id = NodeId(id);
                let geometry = self.geometries.get_mut(id).unwrap();
                if handler.on_click(button, state, pt, node_id, geometry, &mut self.states) {
                    new_focus = Some(node_id);
                    break;
                }
            }
        }

        if self.mouse_focus != new_focus {
            if let Some(prev_focus) = std::mem::replace(&mut self.mouse_focus, new_focus) {
                self.check_id(prev_focus, "Mouse focus has been removed!?");
                self.handlers[prev_focus.index()].on_mouse_focus_lost(prev_focus, &mut self.states);
            }
        }
    }

    #[track_caller]
    fn check_id(&self, id: NodeId, desc: &str) {
        if !self.geometries.contains(id.arena_index()) {
            panic!("Invalid Id({:?}): {}", id, desc);
        }
    }
}

pub fn insert_or_replace<T>(vec: &mut Vec<T>, id: NodeId, item: T) {
    if vec.len() < id.index() {
        vec[id.index()] = item;
    } else {
        vec.insert(id.index(), item);
    }
}

pub fn new_sprite_renderer(texture: UiTextureRegion) -> Box<SpriteRenderer> {
    Box::new(SpriteRenderer {
        texture,
        color: Color::WHITE,
        offset: Point2::new(0.0, 0.0),
        scale: Point2::new(1.0, 1.0),
    })
}

pub trait NodeRenderer {
    fn render(
        &self,
        ui_batch: &mut UiBatch,
        _: &Ui,
        _: NodeId,
        geometry: &NodeGeometry,
        states: &WidgetStates,
    );
}

pub struct EmptyRenderer;

impl NodeRenderer for EmptyRenderer {
    fn render(&self, _: &mut UiBatch, _: &Ui, _: NodeId, _: &NodeGeometry, states: &WidgetStates) {}
}

pub struct SpriteRenderer {
    texture: UiTextureRegion,
    color: Color,
    offset: Point2<f32>,
    scale: Point2<f32>,
}

impl NodeRenderer for SpriteRenderer {
    fn render(
        &self,
        ui_batch: &mut UiBatch,
        _: &Ui,
        _: NodeId,
        geometry: &NodeGeometry,
        _: &WidgetStates,
    ) {
        let pos = Vector4::new(
            geometry.pos.x + self.offset.x,
            geometry.pos.y + self.offset.y,
            geometry.size.x * self.scale.x,
            geometry.size.y * self.scale.y,
        );
        ui_batch.draw(
            self.texture.texture_id,
            &GPUSprite {
                pos,
                uvs: Vector4::new(
                    self.texture.pos.x,
                    self.texture.pos.y,
                    self.texture.size.x,
                    self.texture.size.y,
                ),
                color: Vector4::new(self.color.r, self.color.g, self.color.b, self.color.a),
            },
        );
    }
}

pub struct TextLayout {
    pub offset: Point2<f32>,
    pub width: f32,
    pub height: f32,
    pub color: Color,
    glyphs: Vec<(Point2<f32>, FontGlyph)>,
}

impl TextLayout {
    pub fn new(offset: Point2<f32>, txt: &str, font: &FontMap, color: Color) -> Self {
        let mut glyphs = Vec::new();

        let mut width = 0.0;
        let mut height = 0.0f32;
        let mut last_char = None;

        for (index, c) in txt.chars().enumerate() {
            if c != ' ' {
                // TODO: Handle other whitespace
                let font_char = font.char(c);
                
                if let Some(last_char) = last_char {
                    width += font.pair_kerning(last_char, c);
                }

                glyphs.push((Point2::new(width, font_char.descent), font_char));
                width += font_char.advance_width;
                height = height.max(font_char.height);
                println!("{}: {}", c, width);
            } else {
                width += 20.0; //TODO - add spacing to FontMap
            }

            last_char = Some(c);
        }

        Self {
            width,
            height,
            color,
            offset,
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

pub trait NodeHandler {
    fn on_click(
        &self,
        _: event::MouseButton,
        _: event::ElementState,
        _: Point2<f32>,
        _: NodeId,
        _: &mut NodeGeometry,
        _: &mut WidgetStates,
    ) -> bool {
        false
    }

    fn on_mouse_focus_lost(&self, _: NodeId, _: &mut WidgetStates) {}
}

struct EmptyNodeHandler;

impl NodeHandler for EmptyNodeHandler {}

#[derive(Debug, Clone, Copy)]
pub struct Color {
    pub r: f32,
    pub g: f32,
    pub b: f32,
    pub a: f32,
}

impl Color {
    pub const WHITE: Self = Self {
        r: 1.0,
        g: 1.0,
        b: 1.0,
        a: 1.0,
    };

    pub const BLACK: Self = Self {
        r: 0.0,
        g: 0.0,
        b: 0.0,
        a: 1.0,
    };
}
