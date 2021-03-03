use cgmath::{Point2, Vector4};

use crate::entity::ECS;
use crate::graphics::{FontGlyph, FontMap, NinePatch, TextureRegion2D, UiAssets, UiBatch};
use generational_arena::Arena;
use std::any::Any;
use std::rc::Rc;
use winit::event;

mod button;
mod stack;

mod in_game;

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

pub struct NodeLayout {
    pub min_size: Point2<f32>,
}

impl Default for NodeLayout {
    fn default() -> Self {
        Self {
            min_size: Point2::new(0.0, 0.0),
        }
    }
}

pub type WidgetLayouts = Vec<NodeLayout>;
pub type WidgetGeometries = Arena<NodeGeometry>;
pub type WidgetChildren = Vec<Vec<NodeId>>;
pub type WidgetHandlers = Vec<Box<dyn NodeHandler>>;
pub type EventHandler = Rc<dyn Fn(&mut Ui, &mut ECS)>;

pub struct EventQueue(Vec<EventHandler>);

impl EventQueue {
    pub fn add(&mut self, handler: EventHandler) {
        self.0.push(handler);
    }
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
    geometries: WidgetGeometries,
    layouts: WidgetLayouts,
    parents: Vec<Option<NodeId>>,
    children: WidgetChildren,
    handlers: WidgetHandlers,
    renderers: Vec<Box<dyn NodeRenderer>>,
    states: WidgetStates,
    assets: UiAssets,
    mouse_focus: Option<NodeId>,
    event_queue: EventQueue,
}

impl Ui {
    pub fn new(assets: UiAssets) -> Self {
        let mut ui = Self {
            geometries: Arena::new(),
            layouts: WidgetLayouts::new(),
            parents: Vec::new(),
            children: Vec::new(),
            renderers: Vec::new(),
            handlers: Vec::new(),
            states: WidgetStates { states: Vec::new() },
            mouse_focus: None,
            event_queue: EventQueue(Vec::new()),
            assets,
        };

        in_game::create_in_game_ui(&mut ui);

        ui
    }

    pub fn new_node(
        &mut self,
        parent: Option<NodeId>,
        geometry: NodeGeometry,
        layout: NodeLayout,
        renderer: Box<dyn NodeRenderer>,
        handler: Box<dyn NodeHandler>,
        state: Option<Box<dyn Any>>,
    ) -> NodeId {
        let id = NodeId(self.geometries.insert(geometry));
        insert_or_replace(&mut self.layouts, id, layout);
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
        if !self.is_valid_id(id) {
            return;
        }

        if let Some(parent) = self.parents[id.index()] {
            self.children[parent.index()].retain(|child| *child != id);
        }

        self.geometries.remove(id.arena_index());
        self.layouts[id.index()] = NodeLayout::default();
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

        fn render_all(sprite_batch: &mut UiBatch, ui: &Ui, nodes: &[NodeId]) {
            for node in nodes {
                ui.renderers[node.index()].render(
                    sprite_batch,
                    &ui,
                    *node,
                    &ui.geometries[node.arena_index()],
                    &ui.states,
                );

                render_all(sprite_batch, ui, &ui.children[node.index()]);
            }
        };

        render_all(sprite_batch, &self, &self.find_parentless_nodes());
    }

    pub fn on_click(
        &mut self,
        button: event::MouseButton,
        state: event::ElementState,
        pt: Point2<f32>,
    ) -> bool {
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
                if handler.on_click(
                    button,
                    state,
                    pt,
                    node_id,
                    geometry,
                    &mut self.states,
                    &mut self.event_queue,
                ) {
                    new_focus = Some(node_id);
                    break;
                }
            }
        }

        if self.mouse_focus != new_focus {
            if let Some(prev_focus) = std::mem::replace(&mut self.mouse_focus, new_focus) {
                if self.is_valid_id(prev_focus) {
                    self.handlers[prev_focus.index()]
                        .on_mouse_focus_lost(prev_focus, &mut self.states);
                }
            }
        }

        return new_focus.is_some();
    }

    pub fn update(&mut self, ecs: &mut ECS) {
        let parentless = self.find_parentless_nodes();
        let layout_manager = LayoutManager(&self.children, &self.handlers);

        // TODO: Do not layout every frame
        for id in parentless {
            self.handlers[id.index()].layout(
                &layout_manager,
                id,
                &self.children[id.index()],
                &mut self.geometries,
                &mut self.layouts,
                &mut self.states,
            );
        }

        // Temporarily replace event queue with a zero sized Vec
        let mut events = std::mem::replace(&mut self.event_queue.0, Vec::with_capacity(0));
        events.iter().for_each(|event| (event)(self, ecs));
        events.clear();
        self.event_queue.0 = events;
    }

    fn find_parentless_nodes(&self) -> Vec<NodeId> {
        self.geometries
            .iter()
            .map(|(id, _)| NodeId(id))
            .filter(|id| self.parents[id.index()].is_none())
            .collect()
    }

    fn is_valid_id(&self, id: NodeId) -> bool {
        self.geometries.contains(id.arena_index())
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

#[allow(dead_code)]
pub fn new_sprite_renderer(texture: TextureRegion2D) -> Box<SpriteRenderer> {
    Box::new(SpriteRenderer {
        texture,
        color: Color::WHITE,
        offset: Point2::new(0.0, 0.0),
        scale: Point2::new(1.0, 1.0),
    })
}

pub fn new_ninepatch_renderer(patch: NinePatch) -> Box<NinepatchRenderer> {
    Box::new(NinepatchRenderer {
        patch,
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
    fn render(&self, _: &mut UiBatch, _: &Ui, _: NodeId, _: &NodeGeometry, _: &WidgetStates) {}
}

pub struct SpriteRenderer {
    texture: TextureRegion2D,
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
            pos,
            self.texture,
            Vector4::new(self.color.r, self.color.g, self.color.b, self.color.a),
        );
    }
}

pub struct NinepatchRenderer {
    patch: NinePatch,
    color: Color,
    offset: Point2<f32>,
    scale: Point2<f32>,
}

impl NodeRenderer for NinepatchRenderer {
    fn render(
        &self,
        ui_batch: &mut UiBatch,
        _: &Ui,
        _: NodeId,
        geometry: &NodeGeometry,
        _: &WidgetStates,
    ) {
        let x = geometry.pos.x + self.offset.x;
        let y = geometry.pos.y + self.offset.y;
        let width = geometry.size.x * self.scale.x;
        let height = geometry.size.y * self.scale.y;
        let color = Vector4::new(self.color.r, self.color.g, self.color.b, self.color.a);
        let patch = self.patch;

        let bottom_left_pos =
            Vector4::new(x, y, patch.bottom_left.size.x, patch.bottom_left.size.y);
        let top_left_pos = Vector4::new(
            x,
            y + height - patch.top_left.size.y,
            patch.top_left.size.x,
            patch.top_left.size.y,
        );
        let bottom_right_pos = Vector4::new(
            x + width - patch.bottom_right.size.x,
            y,
            patch.bottom_right.size.x,
            patch.bottom_right.size.y,
        );
        let top_right_pos = Vector4::new(
            x + width - patch.top_right.size.x,
            y + height - patch.top_right.size.y,
            patch.top_right.size.x,
            patch.top_right.size.y,
        );
        let middle_left_pos = Vector4::new(
            x,
            y + bottom_left_pos.w,
            bottom_left_pos.z,
            height - top_left_pos.w - bottom_left_pos.w,
        );
        let middle_right_pos = Vector4::new(
            x + width - bottom_right_pos.z,
            y + bottom_left_pos.w,
            bottom_right_pos.z,
            height - bottom_left_pos.w - top_left_pos.w,
        );
        let bottom_center_pos = Vector4::new(
            x + bottom_left_pos.z,
            y,
            width - bottom_left_pos.z - bottom_right_pos.z,
            bottom_left_pos.w,
        );
        let top_center_pos = Vector4::new(
            x + bottom_left_pos.z,
            y + height - top_left_pos.w,
            width - bottom_left_pos.z - bottom_right_pos.z,
            top_right_pos.w,
        );
        let middle_center_pos = Vector4::new(
            x + bottom_left_pos.z,
            y + bottom_left_pos.w,
            width - middle_left_pos.z - middle_right_pos.z,
            height - bottom_center_pos.w - top_center_pos.w,
        );

        ui_batch.draw(bottom_left_pos, patch.bottom_left, color);
        ui_batch.draw(top_left_pos, patch.top_left, color);
        ui_batch.draw(bottom_right_pos, patch.bottom_right, color);
        ui_batch.draw(top_right_pos, patch.top_right, color);
        ui_batch.draw(middle_left_pos, patch.middle_left, color);
        ui_batch.draw(middle_right_pos, patch.middle_right, color);
        ui_batch.draw(bottom_center_pos, patch.bottom_center, color);
        ui_batch.draw(top_center_pos, patch.top_center, color);
        ui_batch.draw(middle_center_pos, patch.middle_center, color);
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

        for c in txt.chars() {
            if c != ' ' {
                // TODO: Handle other whitespace
                let font_char = font.char(c);

                if let Some(last_char) = last_char {
                    width += font.pair_kerning(last_char, c);
                }

                glyphs.push((Point2::new(width, font_char.descent), font_char));
                width += font_char.advance_width;
                height = height.max(font_char.height);
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
            ui_batch.draw(
                Vector4::new(
                    pos_x + glyph_offset.x,
                    pos_y + glyph_offset.y,
                    glyph.width,
                    glyph.height,
                ),
                glyph.texture,
                color,
            );
        }
    }
}

pub struct LayoutManager<'a>(&'a WidgetChildren, &'a WidgetHandlers);

impl LayoutManager<'_> {
    pub fn layout_all(
        &self,
        nodes: &[NodeId],
        geometries: &mut WidgetGeometries,
        layouts: &mut WidgetLayouts,
        states: &mut WidgetStates,
    ) {
        for node in nodes {
            self.1[node.index()].layout(
                &self,
                *node,
                &self.0[node.index()],
                geometries,
                layouts,
                states,
            )
        }
    }
}

pub trait NodeHandler {
    fn layout<'a>(
        &self,
        _: &'a LayoutManager,
        _: NodeId,
        _: &[NodeId],
        _: &mut WidgetGeometries,
        _: &mut WidgetLayouts,
        _: &mut WidgetStates,
    ) {
    }

    fn on_click(
        &self,
        _: event::MouseButton,
        _: event::ElementState,
        _: Point2<f32>,
        _: NodeId,
        _: &mut NodeGeometry,
        _: &mut WidgetStates,
        _: &mut EventQueue,
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
    #[allow(dead_code)]
    pub const WHITE: Self = Self {
        r: 1.0,
        g: 1.0,
        b: 1.0,
        a: 1.0,
    };

    #[allow(dead_code)]
    pub const BLACK: Self = Self {
        r: 0.0,
        g: 0.0,
        b: 0.0,
        a: 1.0,
    };
}
