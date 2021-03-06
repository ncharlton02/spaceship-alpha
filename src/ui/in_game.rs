use super::{widgets::Button, widgets::Label, *};
use crate::entity::{InputAction, InputManager};
use crate::item::{GameItem, Inventory};

// TODO: Create a container with no size so that
// all of the elements of a scene can be deleted at
// once
pub fn create_in_game_ui(ui: &mut Ui) {
    let top_anchor = layout::WindowAnchor::TopLeft.new(ui);
    let inventory = layout::create_vbox(ui, Some(top_anchor), false);

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
        "Start Mining",
        Rc::new(|_, ecs| ecs.get_resource_mut::<InputManager>().action = InputAction::Mining),
    );
    Button::create(
        ui,
        Some(button_stack),
        "Cancel Input",
        Rc::new(|_, ecs| ecs.get_resource_mut::<InputManager>().action = InputAction::None),
    );
    Button::create(
        ui,
        Some(button_stack),
        "Delete UI",
        Rc::new(move |ui, _| {
            ui.remove_node(button_stack);
        }),
    );
}
