//! Resizable window wrapper for Egui editor.

use std::ops::{BitOr, BitOrAssign};

use egui_baseview::{
	egui::{emath::GuiRounding, CursorIcon, InnerResponse, UiBuilder},
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
	resize_edges: ResizeEdges,
	resize_margin: f32,
}

/// Resize hit zones enabled for a [`ResizableWindow`].
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ResizeEdges(u8);

impl ResizeEdges {
	pub const NONE: Self = Self(0);
	pub const RIGHT: Self = Self(1 << 0);
	pub const BOTTOM: Self = Self(1 << 1);
	pub const BOTTOM_RIGHT: Self = Self(1 << 2);

	#[inline]
	fn contains(self, edge: Self) -> bool {
		self.0 & edge.0 != 0
	}
}

impl BitOr for ResizeEdges {
	type Output = Self;

	fn bitor(self, rhs: Self) -> Self::Output {
		Self(self.0 | rhs.0)
	}
}

impl BitOrAssign for ResizeEdges {
	fn bitor_assign(&mut self, rhs: Self) {
		self.0 |= rhs.0;
	}
}

impl ResizableWindow {
	pub fn new(id_source: impl std::hash::Hash) -> Self {
		Self {
			id: Id::new(id_source),
			min_size: Vec2::splat(16.0),
			resize_edges: ResizeEdges::BOTTOM_RIGHT,
			resize_margin: 14.0,
		}
	}

	/// Won't shrink to smaller than this
	#[inline]
	pub fn min_size(mut self, min_size: impl Into<Vec2>) -> Self {
		self.min_size = min_size.into();
		self
	}

	/// Sets which resize hit zones are active.
	#[inline]
	pub fn resize_edges(mut self, edges: ResizeEdges) -> Self {
		self.resize_edges = edges;
		self
	}

	/// Sets the thickness of the invisible resize hit zones.
	#[inline]
	pub fn resize_margin(mut self, margin: f32) -> Self {
		self.resize_margin = margin.max(1.0);
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

			let margin = self.resize_margin.max(ui.visuals().resize_corner_size);
			let corner_size = Vec2::splat(margin);
			let corner_rect = Rect::from_min_size(ui_rect.max - corner_size, corner_size);

			let mut corner_response = None;

			if self.resize_edges.contains(ResizeEdges::BOTTOM_RIGHT) {
				let response = ui.interact(corner_rect, self.id.with("bottom_right"), Sense::drag());
				handle_resize_zone(ui, egui_state, &response, ResizeDirection::BottomRight, self.min_size);
				corner_response = Some(response);
			}

			if self.resize_edges.contains(ResizeEdges::RIGHT) {
				let right_rect = Rect::from_min_max(
					pos2(ui_rect.right() - margin, ui_rect.top()),
					pos2(ui_rect.right(), corner_rect.top()),
				);
				let response = ui.interact(right_rect, self.id.with("right"), Sense::drag());
				handle_resize_zone(ui, egui_state, &response, ResizeDirection::Right, self.min_size);
			}

			if self.resize_edges.contains(ResizeEdges::BOTTOM) {
				let bottom_rect = Rect::from_min_max(
					pos2(ui_rect.left(), ui_rect.bottom() - margin),
					pos2(corner_rect.left(), ui_rect.bottom()),
				);
				let response = ui.interact(bottom_rect, self.id.with("bottom"), Sense::drag());
				handle_resize_zone(ui, egui_state, &response, ResizeDirection::Bottom, self.min_size);
			}

			if let Some(corner_response) = corner_response {
				paint_resize_corner(&content_ui, &corner_response);
			}

			ret
		})
	}
}

#[derive(Debug, Clone, Copy)]
enum ResizeDirection {
	Right,
	Bottom,
	BottomRight,
}

impl ResizeDirection {
	fn cursor_icon(self) -> CursorIcon {
		match self {
			Self::Right => CursorIcon::ResizeHorizontal,
			Self::Bottom => CursorIcon::ResizeVertical,
			Self::BottomRight => CursorIcon::ResizeNwSe,
		}
	}

	fn desired_size(self, drag: ResizeDrag, screen_pos: egui_baseview::egui::Pos2) -> (f32, f32) {
		let width = match self {
			Self::Right | Self::BottomRight => drag.start_size.0 + screen_pos.x - drag.start_screen_pos.0,
			Self::Bottom => drag.start_size.0,
		};

		let height = match self {
			Self::Bottom | Self::BottomRight => drag.start_size.1 + screen_pos.y - drag.start_screen_pos.1,
			Self::Right => drag.start_size.1,
		};

		(width, height)
	}
}

fn handle_resize_zone(
	ui: &Ui,
	egui_state: &EguiState,
	response: &Response,
	direction: ResizeDirection,
	min_size: Vec2,
) {
	if response.hovered() || response.dragged() {
		ui.output_mut(|output| output.cursor_icon = direction.cursor_icon());
	}

	if response.drag_started() {
		if let Some(screen_pos) = screen_cursor_position(ui.ctx()) {
			let size = egui_state.size();
			egui_state.resize_drag.store(Some(ResizeDrag {
				start_screen_pos: (screen_pos.x, screen_pos.y),
				start_size: (size.0 as f32, size.1 as f32),
				last_requested_size: size,
			}));
		}
	}

	if response.dragged() {
		if let Some(screen_pos) = screen_cursor_position(ui.ctx()) {
			if let Some(mut drag) = egui_state.resize_drag.load() {
				let (desired_width, desired_height) = direction.desired_size(drag, screen_pos);
				let requested_size = (
					desired_width.max(min_size.x).round() as u32,
					desired_height.max(min_size.y).round() as u32,
				);

				if requested_size != drag.last_requested_size {
					drag.last_requested_size = requested_size;
					egui_state.set_requested_size(requested_size);
				}

				egui_state.resize_drag.store(Some(drag));
			} else {
				let size = egui_state.size();
				egui_state.resize_drag.store(Some(ResizeDrag {
					start_screen_pos: (screen_pos.x, screen_pos.y),
					start_size: (size.0 as f32, size.1 as f32),
					last_requested_size: size,
				}));
			}
		}
	}

	if response.drag_stopped() {
		egui_state.resize_drag.store(None);
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
