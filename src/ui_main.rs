use crate::build::*;
use crate::ui_system::UISystem;

use gfx::*;
use imgui::*;

pub fn ui_main(file0: String, file1: String) {
    let bitmap = build(&std::fs::read(file0).expect("Failed to read the RNA file"));

    let mut data = vec![0u8; 4 * 600 * 600];
    let mut i = 0;
    for y in 0..600 {
        for x in 0..600 {
            let Pixel {
                rgb: RGB(r, g, b),
                a,
            } = bitmap.get(Position(x, y));
            data[i] = r;
            data[i + 1] = g;
            data[i + 2] = b;
            data[i + 3] = a;
            i += 4;
        }
    }

    let mut ui_system = UISystem::init("Morph Endo", 1024, 768, 16f32);

    let (_, texture_view) = ui_system
        .rendering_system
        .factory
        .create_texture_immutable_u8::<format::Rgba8>(
            texture::Kind::D2(600, 600, texture::AaMode::Single),
            texture::Mipmap::Provided,
            &[&data],
        )
        .expect("Failed to create a texture");
    let sampler = ui_system
        .rendering_system
        .factory
        .create_sampler(texture::SamplerInfo::new(
            texture::FilterMethod::Bilinear,
            texture::WrapMode::Clamp,
        ));
    let texture_id = ui_system
        .rendering_system
        .renderer
        .textures()
        .insert((texture_view, sampler));

    let mut current = 0;

    ui_system.run(|ui, _continue_| {
        Window::new(im_str!("Main View"))
            .position([0.0, 0.0], Condition::FirstUseEver)
            .size([1200.0, 768.0], Condition::FirstUseEver)
            .movable(false)
            .resizable(false)
            .collapsible(false)
            // .always_auto_resize(true)
            .build(ui, || {
                ChildWindow::new(im_str!("Commands View"))
                    // .position([0f32, 0f32], Condition::FirstUseEver)
                    .size([600f32, 600f32])
                    .movable(false)
                    // .content_size([600f32, 600f32])
                    .build(ui, || {
                        ui.list_box(im_str!("Commands"), &mut current, &[im_str!("a")], 1);
                    });
                ui.same_line(600.0);
                ChildWindow::new(im_str!("Drawing View"))
                    // .position([600f32, 0f32], Condition::FirstUseEver)
                    // .size([600f32, 600f32], Condition::FirstUseEver)
                    .movable(false)
                    .always_auto_resize(true)
                    .build(ui, || {
                        Image::new(texture_id, [600.0, 600.0]).build(ui);
                    });
            });

        // ui.show_demo_window(_continue_);
    });
}
