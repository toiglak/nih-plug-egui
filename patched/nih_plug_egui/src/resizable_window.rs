//! Resizable window wrapper for Egui editor.

use egui_baseview::egui::{CursorIcon, InnerResponse, UiBuilder};

use crate::{
	egui::{pos2, CentralPanel, Context, Id, Rect, Sense, Ui, Vec2},
	EguiState,
};

/// Adds resize handles to the plugin window that can be dragged in order to resize it.
/// Resizing happens through plugin API, hence a custom implementation is needed.
pub struct ResizableWindow {
	id: Id,
	min_size: Vec2,
	handle_margin: f32,
}

impl ResizableWindow {
	pub fn new(id_source: impl std::hash::Hash) -> Self {
		Self {
			id: Id::new(id_source),
			min_size: Vec2::splat(16.0),
			handle_margin: 0.0,
		}
	}

	/// Won't shrink to smaller than this
	#[inline]
	pub fn min_size(mut self, min_size: impl Into<Vec2>) -> Self {
		self.min_size = min_size.into();
		self
	}

	/// Moves the resize handles inwards so they can line up with an inset visual container.
	#[inline]
	pub fn handle_margin(mut self, margin: f32) -> Self {
		self.handle_margin = margin.max(0.0);
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
			let content_rect = ui_rect.shrink(self.handle_margin);
			let mut content_ui = ui.new_child(UiBuilder::new().max_rect(content_rect).layout(*ui.layout()));

			let ret = add_contents(&mut content_ui);

			let corner_size = Vec2::splat(ui.visuals().resize_corner_size.max(28.0));

			for corner in Corner::ALL {
				let response = ui.interact(
					corner.rect(content_rect, corner_size),
					self.id.with(corner.id_suffix()),
					Sense::drag(),
				);

				if response.hovered() || response.dragged() {
					ui.output_mut(|output| output.cursor_icon = corner.cursor_icon());
				}

				if response.drag_started() {
					egui_state.resize_drag_start_size.store(Some(egui_state.size()));
				}

				if response.dragged() {
					let start_size = egui_state
						.resize_drag_start_size
						.load()
						.unwrap_or_else(|| egui_state.size());
					let desired_size = corner
						.size_from_drag(
							Vec2::new(start_size.0 as f32, start_size.1 as f32),
							response.drag_delta(),
						)
						.max(self.min_size);

					egui_state.set_requested_size((desired_size.x.round() as u32, desired_size.y.round() as u32));
				}

				if response.drag_stopped() {
					egui_state.resize_drag_start_size.store(None);
				}
			}

			ret
		})
	}
}

#[derive(Clone, Copy)]
enum Corner {
	TopLeft,
	TopRight,
	BottomLeft,
	BottomRight,
}

impl Corner {
	const ALL: [Self; 4] = [Self::TopLeft, Self::TopRight, Self::BottomLeft, Self::BottomRight];

	fn id_suffix(self) -> &'static str {
		match self {
			Self::TopLeft => "top_left",
			Self::TopRight => "top_right",
			Self::BottomLeft => "bottom_left",
			Self::BottomRight => "bottom_right",
		}
	}

	fn rect(self, bounds: Rect, size: Vec2) -> Rect {
		match self {
			Self::TopLeft => Rect::from_min_size(bounds.min, size),
			Self::TopRight => Rect::from_min_size(pos2(bounds.right() - size.x, bounds.top()), size),
			Self::BottomLeft => Rect::from_min_size(pos2(bounds.left(), bounds.bottom() - size.y), size),
			Self::BottomRight => Rect::from_min_size(bounds.max - size, size),
		}
	}

	fn cursor_icon(self) -> CursorIcon {
		match self {
			Self::TopLeft => CursorIcon::ResizeNorthWest,
			Self::TopRight => CursorIcon::ResizeNorthEast,
			Self::BottomLeft => CursorIcon::ResizeSouthWest,
			Self::BottomRight => CursorIcon::ResizeSouthEast,
		}
	}

	fn size_from_drag(self, start_size: Vec2, delta: Vec2) -> Vec2 {
		match self {
			Self::TopLeft => Vec2::new(start_size.x - delta.x, start_size.y - delta.y),
			Self::TopRight => Vec2::new(start_size.x + delta.x, start_size.y - delta.y),
			Self::BottomLeft => Vec2::new(start_size.x - delta.x, start_size.y + delta.y),
			Self::BottomRight => Vec2::new(start_size.x + delta.x, start_size.y + delta.y),
		}
	}
}
