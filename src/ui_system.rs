use gfx::format::{DepthStencil, Rgba8};
use gfx::handle::{DepthStencilView, RenderTargetView};
use gfx::Encoder;
use gfx_device_gl::*;
use glutin::{
    dpi::LogicalSize, ContextBuilder, Event, EventsLoop, PossiblyCurrent, WindowBuilder,
    WindowEvent, WindowedContext,
};
use imgui::{Context, FontConfig, FontSource, Ui};
use imgui_gfx_renderer::{Renderer, Shaders};
use imgui_winit_support::{HiDpiMode, WinitPlatform};
use std::time::Instant;

// Basic OpenGL-only UI system based on https://github.com/Gekkio/imgui-rs/tree/master/imgui-gfx-examples

pub struct UISystem {
    imgui: Context,
    platform: WinitPlatform,
    pub rendering_system: RenderingSystem,
    events_loop: EventsLoop,
}

pub struct RenderingSystem {
    pub renderer: Renderer<Rgba8, Resources>,
    ctx: WindowedContext<PossiblyCurrent>,
    device: Device,
    pub factory: Factory,
    render_target_view: RenderTargetView<Resources, Rgba8>,
    depth_stencil_view: DepthStencilView<Resources, DepthStencil>,
}

impl UISystem {
    pub fn init(title: &str, width: u32, height: u32, font_size: f32) -> UISystem {
        // ImGui creation
        let mut imgui = Context::create();
        imgui.set_ini_filename(None);

        // Platform creation
        let mut platform = WinitPlatform::init(&mut imgui);

        // ImGui font config
        imgui.fonts().add_font(&[FontSource::DefaultFontData {
            config: Some(FontConfig {
                size_pixels: (font_size as f64 * platform.hidpi_factor()) as f32,
                ..FontConfig::default()
            }),
        }]);
        imgui.io_mut().font_global_scale = (1.0 / platform.hidpi_factor()) as f32;

        // Renderer creation
        let window_builder = WindowBuilder::new()
            .with_title(title.to_string())
            .with_dimensions(LogicalSize::new(width as f64, height as f64));

        let context = ContextBuilder::new().with_vsync(true);
        let events_loop = EventsLoop::new();
        let (ctx_wrapper, device, mut factory, render_target_view, depth_stencil_view) =
            gfx_window_glutin::init(window_builder, context, &events_loop)
                .expect("Failed to initialize gfx_window_glutin");

        let renderer = Renderer::init(&mut imgui, &mut factory, Shaders::GlSl400)
            .expect("Failed to initialize imgui-gfx-renderer");

        platform.attach_window(imgui.io_mut(), ctx_wrapper.window(), HiDpiMode::Rounded);

        UISystem {
            imgui,
            platform,
            rendering_system: RenderingSystem {
                renderer,
                ctx: ctx_wrapper,
                device,
                factory,
                render_target_view,
                depth_stencil_view,
            },
            events_loop,
        }
    }

    pub fn run<F: FnMut(&Ui, &mut bool)>(self, mut build_ui: F) {
        let UISystem {
            mut events_loop,
            mut imgui,
            mut platform,
            mut rendering_system,
        } = self;

        // Approximate fix for ImGui's lack of sRGB support
        for i in 0..imgui.style().colors.len() {
            let src_color = imgui.style().colors[i];
            imgui.style_mut().colors[i][0] = src_color[0].powf(2.2);
            imgui.style_mut().colors[i][1] = src_color[1].powf(2.2);
            imgui.style_mut().colors[i][2] = src_color[2].powf(2.2);
            imgui.style_mut().colors[i][3] = 1.0 - (1.0 - src_color[3]).powf(2.2);
        }

        let mut cmd_buffer: Encoder<_, _> = rendering_system.factory.create_command_buffer().into();

        let mut prev_time = Instant::now();
        let mut continue_ = true;
        while continue_ {
            // Handle events
            events_loop.poll_events(|e| {
                platform.handle_event(imgui.io_mut(), rendering_system.ctx.window(), &e);

                if let Event::WindowEvent { event, .. } = e {
                    match event {
                        WindowEvent::Resized(_) => {
                            gfx_window_glutin::update_views(
                                &rendering_system.ctx,
                                &mut rendering_system.render_target_view,
                                &mut rendering_system.depth_stencil_view,
                            );
                        }
                        WindowEvent::CloseRequested => continue_ = false,
                        _ => {}
                    }
                }
            });

            // Prepare frame
            platform
                .prepare_frame(imgui.io_mut(), rendering_system.ctx.window())
                .expect("Failed to prepare frame");

            // Update time delta
            imgui.io_mut().update_delta_time(prev_time);
            prev_time = Instant::now();

            // Build the UI
            let ui = imgui.frame();
            build_ui(&ui, &mut continue_);

            // Render
            cmd_buffer.clear(&rendering_system.render_target_view, [1.0, 1.0, 1.0, 1.0]);
            platform.prepare_render(&ui, rendering_system.ctx.window());
            let draw_data = ui.render();
            rendering_system
                .renderer
                .render(
                    &mut rendering_system.factory,
                    &mut cmd_buffer,
                    &mut rendering_system.render_target_view,
                    draw_data,
                )
                .expect("Failed to render");
            cmd_buffer.flush(&mut rendering_system.device);

            // Swap buffers
            rendering_system
                .ctx
                .swap_buffers()
                .expect("Failed to swap buffers");
        }
    }
}
