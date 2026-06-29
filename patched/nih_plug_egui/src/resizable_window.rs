//! Resizable window wrapper for Egui editor.

use egui_baseview::{
	egui::{emath::GuiRounding, InnerResponse, UiBuilder},
	screen_cursor_position,
};

use crate::{
	egui::{pos2, CentralPanel, Context, Id, Rect, Response, Sense, Ui, Vec2},
	EguiState, ResizeDrag,
};

/// Adds a corner to the plugin window that can be dragged in order to resize it.
/// Resizing happens through plugin API, hence a custom implementation is needed.
pub struct ResizableWindow {
	id: Id,
	min_size: Vec2,
}

impl ResizableWindow {
	pub fn new(id_source: impl std::hash::Hash) -> Self {
		Self {
			id: Id::new(id_source),
			min_size: Vec2::splat(16.0),
		}
	}

	/// Won't shrink to smaller than this
	#[inline]
	pub fn min_size(mut self, min_size: impl Into<Vec2>) -> Self {
		self.min_size = min_size.into();
		self
	}

	pub fn show<R>(
		self,
		context: &Context,
		egui_state: &EguiState,
		add_contents: impl FnOnce(&mut Ui) -> R,
	) -> InnerResponse<R> {
		CentralPanel::default().show(context, move |ui| {
			let ui_rect = ui.clip_rect();
			let mut content_ui = ui.new_child(UiBuilder::new().max_rect(ui_rect).layout(*ui.layout()));

			let ret = add_contents(&mut content_ui);

			let corner_size = Vec2::splat(ui.visuals().resize_corner_size);
			let corner_rect = Rect::from_min_size(ui_rect.max - corner_size, corner_size);

			let corner_response = ui.interact(corner_rect, self.id.with("corner"), Sense::drag());

			if corner_response.drag_started() {
				if let Some(screen_pos) = screen_cursor_position(ui.ctx()) {
					let size = egui_state.size();
					egui_state.resize_drag.store(Some(ResizeDrag {
						last_screen_pos: (screen_pos.x, screen_pos.y),
						accumulated_size: (size.0 as f32, size.1 as f32),
						last_requested_size: size,
					}));
				}
			}

			if corner_response.dragged() {
				if let Some(screen_pos) = screen_cursor_position(ui.ctx()) {
					if let Some(mut drag) = egui_state.resize_drag.load() {
						let delta_x = screen_pos.x - drag.last_screen_pos.0;
						let delta_y = screen_pos.y - drag.last_screen_pos.1;

						drag.last_screen_pos = (screen_pos.x, screen_pos.y);
						drag.accumulated_size.0 = (drag.accumulated_size.0 + delta_x).max(self.min_size.x);
						drag.accumulated_size.1 = (drag.accumulated_size.1 + delta_y).max(self.min_size.y);

						let requested_size = (
							drag.accumulated_size.0.round() as u32,
							drag.accumulated_size.1.round() as u32,
						);

						if requested_size != drag.last_requested_size {
							drag.last_requested_size = requested_size;
							egui_state.set_requested_size(requested_size);
						}

						egui_state.resize_drag.store(Some(drag));
					} else {
						let size = egui_state.size();
						egui_state.resize_drag.store(Some(ResizeDrag {
							last_screen_pos: (screen_pos.x, screen_pos.y),
							accumulated_size: (size.0 as f32, size.1 as f32),
							last_requested_size: size,
						}));
					}
				}
			}

			if corner_response.drag_stopped() {
				egui_state.resize_drag.store(None);
			}

			paint_resize_corner(&content_ui, &corner_response);

			ret
		})
	}
}

pub fn paint_resize_corner(ui: &Ui, response: &Response) {
	let stroke = ui.style().interact(response).fg_stroke;

	let painter = ui.painter();
	let rect = response.rect.translate(-Vec2::splat(2.0)); // move away from the corner
	let cp = rect.max.round_to_pixels(painter.pixels_per_point());

	let mut w = 2.0;

	while w <= rect.width() && w <= rect.height() {
		painter.line_segment([pos2(cp.x - w, cp.y), pos2(cp.x, cp.y - w)], stroke);
		w += 4.0;
	}
}
