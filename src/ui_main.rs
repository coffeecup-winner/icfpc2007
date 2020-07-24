use crate::ui_system::UISystem;

pub fn ui_main(file0: String, file1: String) {
    let ui_system = UISystem::init("Morph Endo", 1024, 768, 14f32);
    ui_system.run(|ui, continue_| {
        ui.show_demo_window(continue_);
    });
}
