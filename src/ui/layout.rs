use super::*;
use cgmath::Point2;

enum BoxLayoutManager {
    HBox,
    VBox,
}

impl BoxLayoutManager {
    fn major_minor_to_xy(&self, major: f32, minor: f32) -> Point2<f32> {
        match self {
            Self::HBox => Point2::new(major, minor),
            Self::VBox => Point2::new(minor, major),
        }
    }
    fn set_major_axis(&self, p: &mut Point2<f32>, val: f32) {
        match self {
            Self::HBox => p.x = val,
            Self::VBox => p.y = val,
        }
    }

    fn set_minor_axis(&self, p: &mut Point2<f32>, val: f32) {
        match self {
            Self::HBox => p.y = val,
            Self::VBox => p.x = val,
        }
    }

    fn major_axis(&self, p: &Point2<f32>) -> f32 {
        match self {
            Self::HBox => p.x,
            Self::VBox => p.y,
        }
    }

    fn minor_axis(&self, p: &Point2<f32>) -> f32 {
        match self {
            Self::HBox => p.y,
            Self::VBox => p.x,
        }
    }
}

impl NodeHandler for BoxLayoutManager {
    fn layout<'a>(
        &self,
        layout_manager: &'a LayoutManager,
        node: NodeId,
        children: &[NodeId],
        geometries: &mut WidgetGeometries,
        layouts: &mut WidgetLayouts,
        states: &mut WidgetStates,
    ) {
        let spacing: f32 = 5.0;
        let mut major_axis = spacing;
        let mut minor_axis: f32 = 0.0;

        layout_manager.layout_all(children, geometries, layouts, states);

        for child in children {
            let min_size = layouts.get(child.index()).unwrap().min_size;
            minor_axis = minor_axis.max(self.minor_axis(&min_size));
            major_axis += self.major_axis(&min_size) + spacing;
        }

        let mut pos = geometries[node.arena_index()].pos;
        pos.x += spacing;
        pos.y += spacing;
        for child in children {
            let geometry = &mut geometries[child.arena_index()];
            self.set_minor_axis(&mut geometry.pos, self.minor_axis(&pos));
            self.set_major_axis(&mut geometry.pos, self.major_axis(&pos));
            self.set_minor_axis(&mut geometry.size, minor_axis);

            let major_pos = self.major_axis(&geometry.size) + self.major_axis(&pos) + spacing;
            self.set_major_axis(&mut pos, major_pos);
        }

        minor_axis += spacing * 2.0;
        let size = self.major_minor_to_xy(major_axis, minor_axis);
        layouts[node.index()].min_size = size;
        geometries[node.arena_index()].size = size;
    }
}

pub fn create_vbox(ui: &mut Ui, parent: Option<NodeId>, draw_background: bool) -> NodeId {
    ui.new_node(
        parent,
        NodeGeometry {
            pos: Point2::new(0.0, 0.0),
            size: Point2::new(0.0, 0.0),
        },
        NodeLayout::default(),
        if draw_background {
            new_ninepatch_renderer(ui.assets.pane)
        } else {
            Box::new(EmptyRenderer)
        },
        Box::new(BoxLayoutManager::VBox),
        None,
    )
}

pub fn create_hbox(ui: &mut Ui, parent: Option<NodeId>, draw_background: bool) -> NodeId {
    ui.new_node(
        parent,
        NodeGeometry {
            pos: Point2::new(0.0, 0.0),
            size: Point2::new(0.0, 0.0),
        },
        NodeLayout::default(),
        if draw_background {
            new_ninepatch_renderer(ui.assets.pane)
        } else {
            Box::new(EmptyRenderer)
        },
        Box::new(BoxLayoutManager::HBox),
        None,
    )
}

pub enum WindowAnchor {
    // TODO: Add the rest of the variants
    TopLeft,
    BottomLeft,
}

impl WindowAnchor {
    pub fn new(self, ui: &mut Ui) -> NodeId {
        ui.new_node(
            None,
            NodeGeometry {
                pos: Point2::new(0.0, 0.0),
                size: Point2::new(0.0, 0.0),
            },
            NodeLayout::default(),
            Box::new(EmptyRenderer),
            Box::new(self),
            None,
        )
    }
}

impl NodeHandler for WindowAnchor {
    fn layout<'a>(
        &self,
        layout_manager: &'a LayoutManager,
        _: NodeId,
        children: &[NodeId],
        geometries: &mut WidgetGeometries,
        layouts: &mut WidgetLayouts,
        states: &mut WidgetStates,
    ) {
        layout_manager.layout_all(children, geometries, layouts, states);
        let mut layout = |func: fn(&mut NodeGeometry, Point2<f32>)| {
            for child in children {
                let geometry = &mut geometries[child.arena_index()];
                (func)(geometry, layout_manager.window_size);
            }
        };

        match self {
            Self::BottomLeft => layout(|geometry, _| {
                geometry.pos.x = 0.0;
                geometry.pos.y = 0.0;
            }),
            Self::TopLeft => layout(|geometry, window_size| {
                geometry.pos.x = 0.0;
                geometry.pos.y = window_size.y - geometry.size.y;
            }),
        }
    }
}
