use super::{widgets::Button, widgets::Label, *};

pub fn create_end_game(ui: &mut Ui, time: u64) -> NodeId {
    let scene = layout::create_scene(ui);

    let center_anchor = layout::WindowAnchor::Center.new(ui, scene);
    let vbox = layout::create_vbox(ui, Some(center_anchor), false);

    Button::create(
        ui,
        Some(vbox),
        "Quit",
        Rc::new(|_, _| std::process::exit(0)),
    );
    Label::create(ui, Some(vbox), &format!("Time: {}", time));
    Label::create(ui, Some(vbox), "You lose!");
    widgets::create_texture_box(ui, Some(vbox), ui.assets.colin);

    scene
}
