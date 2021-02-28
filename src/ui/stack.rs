use super::*;
use cgmath::Point2;

struct StackHandler;

impl NodeHandler for StackHandler {
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
        let mut width: f32 = 0.0;
        let mut height = spacing;

        layout_manager.layout_all(children, geometries, layouts, states);

        for child in children {
            let min_size = layouts.get(child.index()).unwrap().min_size;
            width = width.max(min_size.x);
            height += min_size.y + spacing;
        }

        let mut pos = geometries[node.arena_index()].pos;
        pos.x += spacing;
        pos.y += spacing;
        for child in children {
            let mut geometry = &mut geometries[child.arena_index()];
            geometry.pos.x = pos.x;
            geometry.pos.y = pos.y;
            geometry.size.x = width;
            pos.y += geometry.size.y + spacing;
        }

        width += spacing * 2.0;
        layouts[node.index()].min_size = Point2::new(width, height);
        geometries[node.arena_index()].size = Point2::new(width, height);
    }
}

pub fn create_stack(ui: &mut Ui, parent: Option<NodeId>) -> NodeId {
    ui.new_node(
        parent,
        NodeGeometry {
            pos: Point2::new(0.0, 0.0),
            size: Point2::new(0.0, 0.0),
        },
        NodeLayout::default(),
        new_ninepatch_renderer(ui.assets.pane),
        Box::new(StackHandler),
        None,
    )
}
