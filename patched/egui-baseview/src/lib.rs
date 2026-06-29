mod renderer;
mod translate;
mod window;

pub use window::{EguiWindow, Queue};

pub use egui;
pub use renderer::GraphicsConfig;

pub fn screen_cursor_position(ctx: &egui::Context) -> Option<egui::Pos2> {
	ctx.data_mut(|data| data.get_temp(screen_cursor_position_id()))
}

pub(crate) fn screen_cursor_position_id() -> egui::Id {
	egui::Id::new("__egui_baseview_screen_cursor_position")
}
