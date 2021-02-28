use super::*;
use crate::entity::{InputAction, InputManager};

// TODO: Create a container with no size so that
// all of the elements of a scene can be deleted at
// once
pub fn create_in_game_ui(ui: &mut Ui) {
    let stack = stack::create_stack(ui, None);
    button::create_button(
        ui,
        Some(stack),
        "Start Laser",
        Rc::new(|_, ecs| ecs.get_resource_mut::<InputManager>().action = InputAction::Laser),
    );
    button::create_button(
        ui,
        Some(stack),
        "Start Mining",
        Rc::new(|_, ecs| ecs.get_resource_mut::<InputManager>().action = InputAction::Mining),
    );
    button::create_button(
        ui,
        Some(stack),
        "Cancel Input",
        Rc::new(|_, ecs| ecs.get_resource_mut::<InputManager>().action = InputAction::None),
    );
    button::create_button(
        ui,
        Some(stack),
        "Delete UI",
        Rc::new(move |ui, _| {
            ui.remove_node(stack);
        }),
    );
}
