use super::{widgets::Button, widgets::Label, *};
use crate::block::Blocks;
use crate::entity::{InputAction, InputManager};
use crate::item::{GameItem, Inventory};

// TODO: Create a container with no size so that
// all of the elements of a scene can be deleted at
// once
pub fn create_in_game_ui(ui: &mut Ui) {
    let top_left_anchor = layout::WindowAnchor::TopLeft.new(ui);
    let inventory = layout::create_vbox(ui, Some(top_left_anchor), false);

    for item in GameItem::iter() {
        let hbox = layout::create_hbox(ui, Some(inventory), false);
        let texture = *ui
            .assets
            .item_icons
            .get(item)
            .expect(&format!("No texture for item: {:?}", item));

        widgets::create_texture_box(ui, Some(hbox), texture);
        let label = Label::create(ui, Some(hbox), &format!("{:?}: 0", item));
        ui.set_on_update(
            label,
            Rc::new(move |ui, ecs| {
                let inventory = ecs.get_resource::<Inventory>();
                Label::update_text(
                    ui,
                    label,
                    &format!("{:?}: {}", item, inventory.amount(item)),
                );
            }),
        );
    }

    let top_anchor = layout::WindowAnchor::TopCenter.new(ui);
    let action_label = Label::create(ui, Some(top_anchor), "Current Action: None");
    ui.set_on_update(
        action_label,
        Rc::new(move |ui, ecs| {
            let input_action = &ecs
                .get_resource::<crate::entity::input::InputManager>()
                .action;
            let action_text = input_action.display_name(&ecs);
            Label::update_text(
                ui,
                action_label,
                &format!("Current Action: {}", action_text),
            );
        }),
    );

    let button_stack = layout::create_vbox(ui, None, true);
    Button::create(
        ui,
        Some(button_stack),
        "Start Laser",
        Rc::new(|_, ecs| ecs.get_resource_mut::<InputManager>().action = InputAction::Laser),
    );
    Button::create(
        ui,
        Some(button_stack),
        "Build Laser",
        Rc::new(|_, ecs| {
            let blocks = ecs.get_resource::<Blocks>();
            ecs.get_resource_mut::<InputManager>().action = InputAction::Build(blocks.laser);
        }),
    );
    Button::create(
        ui,
        Some(button_stack),
        "Build Cooler",
        Rc::new(|_, ecs| {
            let blocks = ecs.get_resource::<Blocks>();
            ecs.get_resource_mut::<InputManager>().action = InputAction::Build(blocks.cooler);
        }),
    );

    create_heat_bar(ui);
}

struct HeatBarState {
    bar_percent: f32,
}

struct HeatBarRenderer;

impl NodeRenderer for HeatBarRenderer {
    fn render(
        &self,
        ui_batch: &mut UiBatch,
        _: &Ui,
        node_id: NodeId,
        geometry: &NodeGeometry,
        states: &WidgetStates,
    ) {
        // TODO: Rewrite in terms of padding boxes?
        let percent = states.get::<HeatBarState>(node_id).unwrap().bar_percent;
        let outer_color = Color::WHITE.as_vec4();
        let inner_color = Color::RED.as_vec4();
        let padding = 4.0;

        let outer_bar = Vector4::new(
            geometry.pos.x,
            geometry.pos.y,
            geometry.size.x,
            geometry.size.y,
        );
        let inner_bar = Vector4::new(
            geometry.pos.x + padding,
            geometry.pos.y + padding,
            (geometry.size.x - (padding * 2.0)) * percent,
            geometry.size.y - (padding * 2.0),
        );

        ui_batch.rect(outer_bar, outer_color);
        ui_batch.rect(inner_bar, inner_color);
    }
}

pub fn create_heat_bar(ui: &mut Ui) {
    let bottom_center_anchor = layout::WindowAnchor::BottomCenter.new(ui);
    let heat_bar_size = Point2::new(250.0, 50.0);
    let stack = layout::create_stack(ui, Some(bottom_center_anchor));

    let bar = ui.new_node(
        Some(stack),
        NodeGeometry {
            pos: Point2::new(0.0, 0.0),
            size: heat_bar_size,
        },
        NodeLayout {
            min_size: heat_bar_size,
        },
        Box::new(HeatBarRenderer),
        Box::new(EmptyNodeHandler),
        Some(Box::new(HeatBarState { bar_percent: 0.5 })),
    );
    let label = Label::create(ui, Some(stack), "Heat: 0");
    Label::set_color(ui, label, Color::BLACK);

    ui.set_on_update(
        label,
        Rc::new(move |ui, ecs| {
            use crate::entity::ship;
            let ships = ecs.get_component::<ship::Ship>();
            let ship = ships.get(ecs.player_ship).expect("Player ship invalid!");
            let percent = ship.heat / ship::MAX_HEAT;

            Label::update_text(
                ui,
                label,
                &format!("Heat: {:.0} / {}", ship.heat, ship::MAX_HEAT),
            );
            ui.states_mut()
                .get_mut::<HeatBarState>(bar)
                .unwrap()
                .bar_percent = percent;
        }),
    );
}
