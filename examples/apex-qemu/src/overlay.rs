use std::sync::Once;
use std::thread::sleep;
use std::time::Duration;

use crossbeam_channel::Sender;
use egui_backend::WindowBackend;
use egui_render_three_d::ThreeDBackend as DefaultGfxBackend;

use egui_overlay::EguiOverlay;

fn setup_custom_fonts(ctx: &egui::Context) {
    // Start with the default fonts (we will be adding to them rather than replacing them).
    let mut fonts = egui::FontDefinitions::default();

    // Install my own font (maybe supporting non-latin characters).
    // .ttf and .otf files supported.
    fonts.font_data.insert(
        "Noto_Sans_SC".to_owned(),
        egui::FontData::from_static(include_bytes!(
            "../fonts/Noto_Sans_SC/NotoSansSC-VariableFont_wght.ttf"
        )),
    );

    // Put my font first (highest priority) for proportional text:
    fonts
        .families
        .entry(egui::FontFamily::Proportional)
        .or_default()
        .insert(0, "Noto_Sans_SC".to_owned());

    // Put my font as last fallback for monospace:
    fonts
        .families
        .entry(egui::FontFamily::Monospace)
        .or_default()
        .push("Noto_Sans_SC".to_owned());

    // Tell egui to use these fonts:
    ctx.set_fonts(fonts);
}

pub fn get_context(sx: Sender<egui_backend::egui::Context>) {
    egui_overlay::start(DrawFrame { sx });
}


pub struct DrawFrame {
    pub sx: Sender<egui_backend::egui::Context>,
}

impl EguiOverlay for crate::overlay::DrawFrame {
    fn gui_run(
        &mut self,
        egui_context: &egui_backend::egui::Context,
        _default_gfx_backend: &mut DefaultGfxBackend,
        glfw_backend: &mut egui_window_glfw_passthrough::GlfwBackend,
    ) {
        static ONCE: Once = Once::new();

        ONCE.call_once(|| {
            glfw_backend.set_window_position([0f32; 2]);
            glfw_backend.set_window_size([2560.0f32 - 1.0, 1440.0f32 - 1.0]);
            self.sx.send(egui_context.clone()).unwrap();
        });


        // here you decide if you want to be passthrough or not.
        if egui_context.wants_pointer_input() || egui_context.wants_keyboard_input() {
            glfw_backend.window.set_mouse_passthrough(false);
        } else {
            glfw_backend.window.set_mouse_passthrough(true);
        }
        sleep(Duration::from_millis(20));
        egui_context.request_repaint();
    }
}
