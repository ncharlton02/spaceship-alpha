use super::*;
use crate::entity::{InputAction, InputManager, ECS};

// TODO: Create a container with no size so that
// all of the elements of a scene can be deleted at
// once
pub fn create_in_game_ui(ui: &mut Ui) {
    let stack = stack::create_stack(ui, None);
    let b1 = button::create_button(
        ui,
        Some(stack),
        "Start Laser",
        Rc::new(|_, ecs| ecs.get_resource_mut::<InputManager>().action = InputAction::Laser),
    );
    let b2 = button::create_button(
        ui,
        Some(stack),
        "Start Mining",
        Rc::new(|_, ecs| ecs.get_resource_mut::<InputManager>().action = InputAction::Mining),
    );
    let b3 = button::create_button(
        ui,
        Some(stack),
        "Cancel Input",
        Rc::new(|_, ecs| ecs.get_resource_mut::<InputManager>().action = InputAction::None),
    );
    let clear_ui = button::create_button(
        ui,
        Some(stack),
        "ClearUI",
        Rc::new(move |ui, _| {
            ui.remove_node(b1);
            ui.remove_node(b2);
            ui.remove_node(b3);
        }),
    );
}
