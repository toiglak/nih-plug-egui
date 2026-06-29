use nih_plug_egui::egui::{
	self, Color32, FontId, InnerResponse, Response, RichText, Sense, Stroke, TextStyle, Ui, Vec2, Widget,
};

pub mod palette {
	use nih_plug_egui::egui::Color32;

	pub const APP_BG: Color32 = Color32::from_rgb(10, 13, 18);
	pub const PANEL_BG: Color32 = Color32::from_rgb(14, 18, 25);
	pub const CARD_BG: Color32 = Color32::from_rgb(18, 23, 31);
	pub const CARD_BG_ALT: Color32 = Color32::from_rgb(21, 28, 38);
	pub const CARD_STROKE: Color32 = Color32::from_rgb(43, 54, 70);
	pub const CARD_STROKE_HOVER: Color32 = Color32::from_rgb(62, 83, 112);
	pub const TEXT: Color32 = Color32::from_rgb(232, 238, 246);
	pub const MUTED: Color32 = Color32::from_rgb(145, 156, 174);
	pub const SUBTLE: Color32 = Color32::from_rgb(94, 106, 126);
	pub const ACCENT: Color32 = Color32::from_rgb(36, 211, 238);
	pub const ACCENT_GREEN: Color32 = Color32::from_rgb(151, 230, 72);
	pub const ACCENT_ORANGE: Color32 = Color32::from_rgb(249, 137, 43);
	pub const BUTTON_BG: Color32 = Color32::from_rgb(30, 39, 53);
	pub const BUTTON_HOVER: Color32 = Color32::from_rgb(39, 52, 72);
	pub const INPUT_BG: Color32 = Color32::from_rgb(8, 11, 16);
}

pub fn apply_theme(egui_ctx: &egui::Context) {
	let mut style = (*egui_ctx.style()).clone();
	style.spacing.item_spacing = Vec2::new(10.0, 8.0);
	style.spacing.button_padding = Vec2::new(10.0, 7.0);
	style.spacing.window_margin = egui::Margin::same(0);
	style.spacing.slider_width = 180.0;
	// Keep egui's default scroll input path. Forcing raw wheel deltas removes
	// trackpad momentum, but it makes mouse-wheel scrolling visibly jagged.
	style.visuals = egui::Visuals::dark();
	style.visuals.panel_fill = palette::APP_BG;
	style.visuals.extreme_bg_color = palette::INPUT_BG;
	style.visuals.faint_bg_color = palette::CARD_BG_ALT;
	style.visuals.window_fill = palette::PANEL_BG;
	style.visuals.window_corner_radius = egui::CornerRadius::same(8);
	style.visuals.menu_corner_radius = egui::CornerRadius::same(8);
	style.visuals.selection.bg_fill = Color32::from_rgb(18, 113, 140);
	style.visuals.selection.stroke = Stroke::new(1.0, palette::ACCENT);
	style.visuals.hyperlink_color = palette::ACCENT;
	style.visuals.slider_trailing_fill = true;
	style.visuals.button_frame = true;

	for visuals in [
		&mut style.visuals.widgets.noninteractive,
		&mut style.visuals.widgets.inactive,
		&mut style.visuals.widgets.hovered,
		&mut style.visuals.widgets.active,
		&mut style.visuals.widgets.open,
	] {
		visuals.corner_radius = egui::CornerRadius::same(6);
		visuals.fg_stroke = Stroke::new(1.0, palette::TEXT);
	}

	style.visuals.widgets.noninteractive.bg_fill = palette::CARD_BG;
	style.visuals.widgets.noninteractive.bg_stroke = Stroke::new(1.0, palette::CARD_STROKE);
	style.visuals.widgets.inactive.bg_fill = palette::BUTTON_BG;
	style.visuals.widgets.inactive.bg_stroke = Stroke::new(1.0, palette::CARD_STROKE);
	style.visuals.widgets.hovered.bg_fill = palette::BUTTON_HOVER;
	style.visuals.widgets.hovered.bg_stroke = Stroke::new(1.0, palette::CARD_STROKE_HOVER);
	style.visuals.widgets.active.bg_fill = Color32::from_rgb(23, 75, 92);
	style.visuals.widgets.active.bg_stroke = Stroke::new(1.0, palette::ACCENT);

	style.text_styles.insert(TextStyle::Heading, FontId::proportional(24.0));
	style.text_styles.insert(TextStyle::Body, FontId::proportional(14.0));
	style.text_styles.insert(TextStyle::Button, FontId::proportional(13.0));
	style.text_styles.insert(TextStyle::Small, FontId::proportional(11.0));

	egui_ctx.set_style(style);
}

pub struct Card<'a> {
	title: &'a str,
}

impl<'a> Card<'a> {
	pub fn new(title: &'a str) -> Self {
		Self { title }
	}

	pub fn show<R>(self, ui: &mut Ui, body: impl FnOnce(&mut Ui) -> R) -> InnerResponse<R> {
		let inner = egui::Frame::default()
			.fill(palette::CARD_BG)
			.stroke(Stroke::new(1.0, palette::CARD_STROKE))
			.corner_radius(8)
			.inner_margin(egui::Margin::same(14))
			.show(ui, |ui| {
				ui.set_width(ui.available_width());
				ui.label(RichText::new(self.title).size(15.0).strong().color(palette::TEXT));
				ui.add_space(8.0);
				body(ui)
			});
		ui.add_space(14.0);
		inner
	}
}

pub struct Pill {
	label: String,
	value: String,
	accent: Color32,
}

impl Pill {
	pub fn new(label: impl Into<String>, value: impl Into<String>, accent: Color32) -> Self {
		Self {
			label: label.into(),
			value: value.into(),
			accent,
		}
	}
}

impl Widget for Pill {
	fn ui(self, ui: &mut Ui) -> Response {
		egui::Frame::default()
			.fill(Color32::from_rgb(19, 27, 37))
			.stroke(Stroke::new(1.0, Color32::from_rgb(46, 61, 80)))
			.corner_radius(8)
			.inner_margin(egui::Margin::symmetric(10, 6))
			.show(ui, |ui| {
				ui.horizontal(|ui| {
					ui.label(RichText::new(self.label).size(10.0).color(palette::SUBTLE).strong());
					ui.label(RichText::new(self.value).size(12.0).color(self.accent).strong());
				});
			})
			.response
	}
}

pub struct StatusRow {
	label: String,
	value: String,
}

impl StatusRow {
	pub fn new(label: impl Into<String>, value: impl Into<String>) -> Self {
		Self {
			label: label.into(),
			value: value.into(),
		}
	}
}

impl Widget for StatusRow {
	fn ui(self, ui: &mut Ui) -> Response {
		egui::Frame::default()
			.fill(Color32::from_rgb(15, 20, 28))
			.corner_radius(6)
			.inner_margin(egui::Margin::symmetric(9, 6))
			.show(ui, |ui| {
				ui.horizontal(|ui| {
					ui.label(RichText::new(self.label).size(11.0).color(palette::SUBTLE).strong());
					ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
						ui.label(RichText::new(self.value).size(11.0).color(palette::MUTED));
					});
				});
			})
			.response
	}
}

pub struct ActionButton {
	label: String,
	min_size: Vec2,
	corner_radius: u8,
}

impl ActionButton {
	pub fn new(label: impl Into<String>) -> Self {
		Self {
			label: label.into(),
			min_size: Vec2::new(54.0, 32.0),
			corner_radius: 6,
		}
	}

	pub fn min_size(mut self, min_size: Vec2) -> Self {
		self.min_size = min_size;
		self
	}

	pub fn corner_radius(mut self, radius: u8) -> Self {
		self.corner_radius = radius;
		self
	}
}

impl Widget for ActionButton {
	fn ui(self, ui: &mut Ui) -> Response {
		ui.add(
			egui::Button::new(RichText::new(self.label).size(13.0).color(palette::TEXT).strong())
				.fill(palette::BUTTON_BG)
				.stroke(Stroke::new(1.0, palette::CARD_STROKE))
				.corner_radius(self.corner_radius)
				.min_size(self.min_size),
		)
	}
}

pub struct NormalizedSlider<'a> {
	value: &'a mut f32,
}

impl<'a> NormalizedSlider<'a> {
	pub fn new(value: &'a mut f32) -> Self {
		*value = value.clamp(0.0, 1.0);
		Self { value }
	}
}

impl Widget for NormalizedSlider<'_> {
	fn ui(self, ui: &mut Ui) -> Response {
		let desired_size = Vec2::new(ui.available_width(), 28.0);
		let (rect, mut response) = ui.allocate_exact_size(desired_size, Sense::click_and_drag());
		let painter = ui.painter_at(rect);
		let track_rect = egui::Rect::from_min_max(
			egui::pos2(rect.left(), rect.center().y - 4.0),
			egui::pos2(rect.right(), rect.center().y + 4.0),
		);
		if let Some(position) = response
			.interact_pointer_pos()
			.filter(|_| response.clicked() || response.dragged())
		{
			*self.value = ((position.x - rect.left()) / rect.width()).clamp(0.0, 1.0);
			response.mark_changed();
		}
		let fill_rect = egui::Rect::from_min_max(
			track_rect.left_top(),
			egui::pos2(
				track_rect.left() + track_rect.width() * *self.value,
				track_rect.bottom(),
			),
		);

		painter.rect_filled(track_rect, 5.0, Color32::from_rgb(33, 42, 55));
		painter.rect_filled(fill_rect, 5.0, palette::ACCENT);
		let handle_x = track_rect.left() + track_rect.width() * *self.value;
		painter.circle_filled(egui::pos2(handle_x, track_rect.center().y), 7.0, palette::TEXT);
		painter.circle_stroke(
			egui::pos2(handle_x, track_rect.center().y),
			7.0,
			Stroke::new(2.0, palette::ACCENT),
		);

		response
	}
}

pub struct SegmentedMeter<'a> {
	value: &'a mut f32,
	steps: usize,
	height: f32,
}

impl<'a> SegmentedMeter<'a> {
	pub fn new(value: &'a mut f32) -> Self {
		*value = value.clamp(0.0, 1.0);
		Self {
			value,
			steps: 24,
			height: 86.0,
		}
	}

	pub fn steps(mut self, steps: usize) -> Self {
		self.steps = steps.max(2);
		self
	}
}

impl Widget for SegmentedMeter<'_> {
	fn ui(self, ui: &mut Ui) -> Response {
		let desired_size = Vec2::new(ui.available_width(), self.height);
		let (rect, mut response) = ui.allocate_exact_size(desired_size, Sense::click_and_drag());
		let painter = ui.painter_at(rect);
		let gap = 3.0;
		let step_width = ((rect.width() - gap * (self.steps - 1) as f32) / self.steps as f32).max(4.0);
		if let Some(position) = response
			.interact_pointer_pos()
			.filter(|_| response.clicked() || response.dragged())
		{
			*self.value = ((position.x - rect.left()) / rect.width()).clamp(0.0, 1.0);
			response.mark_changed();
		}

		for step in 0..self.steps {
			let step_normalized = step as f32 / (self.steps - 1) as f32;
			let height = 18.0 + 52.0 * step_normalized;
			let left = rect.left() + step as f32 * (step_width + gap);
			let bottom = rect.bottom() - 8.0;
			let bar_rect = egui::Rect::from_min_size(egui::pos2(left, bottom - height), Vec2::new(step_width, height));
			let color = if step_normalized <= *self.value {
				active_meter_color(step_normalized)
			} else {
				Color32::from_rgb(37, 46, 62)
			};
			painter.rect_filled(bar_rect, 3.0, color);
		}

		response
	}
}

pub struct AnimationBars {
	frame_count: u64,
	gain: f32,
}

impl AnimationBars {
	pub fn new(frame_count: u64, gain: f32) -> Self {
		Self { frame_count, gain }
	}
}

impl Widget for AnimationBars {
	fn ui(self, ui: &mut Ui) -> Response {
		let desired_size = Vec2::new(ui.available_width(), 72.0);
		let (rect, response) = ui.allocate_exact_size(desired_size, Sense::hover());
		let painter = ui.painter_at(rect);
		let phase = self.frame_count as f32 * 0.08;
		painter.rect_filled(rect, 8.0, Color32::from_rgb(12, 17, 24));

		for i in 0..12 {
			let wave = (phase + i as f32 * 0.45).sin().abs();
			let height = 10.0 + 48.0 * wave * self.gain.max(0.15);
			let x = rect.left() + 12.0 + i as f32 * 15.0;
			let bar = egui::Rect::from_min_size(egui::pos2(x, rect.bottom() - height - 8.0), Vec2::new(7.0, height));
			let color = match i % 3 {
				0 => palette::ACCENT,
				1 => palette::ACCENT_GREEN,
				_ => palette::ACCENT_ORANGE,
			};
			painter.rect_filled(bar, 4.0, color);
		}

		response
	}
}

pub struct HoverPad<'a> {
	label: &'a str,
	cursor: egui::CursorIcon,
	width: f32,
}

impl<'a> HoverPad<'a> {
	pub fn new(label: &'a str, cursor: egui::CursorIcon, width: f32) -> Self {
		Self { label, cursor, width }
	}
}

impl Widget for HoverPad<'_> {
	fn ui(self, ui: &mut Ui) -> Response {
		let (rect, response) = ui.allocate_exact_size(Vec2::new(self.width, 44.0), Sense::hover());
		let color = if response.hovered() {
			ui.output_mut(|output| output.cursor_icon = self.cursor);
			Color32::from_rgb(31, 48, 71)
		} else {
			Color32::from_rgb(18, 26, 38)
		};

		ui.painter().rect(
			rect,
			8.0,
			color,
			Stroke::new(
				1.0,
				if response.hovered() {
					palette::CARD_STROKE_HOVER
				} else {
					palette::CARD_STROKE
				},
			),
			egui::StrokeKind::Inside,
		);
		ui.painter().text(
			rect.center(),
			egui::Align2::CENTER_CENTER,
			self.label,
			FontId::proportional(12.0),
			palette::TEXT,
		);

		response
	}
}

pub struct LazyRow {
	index: usize,
	selected: bool,
}

impl LazyRow {
	pub fn new(index: usize, selected: bool) -> Self {
		Self { index, selected }
	}
}

impl Widget for LazyRow {
	fn ui(self, ui: &mut Ui) -> Response {
		let (rect, response) = ui.allocate_exact_size(Vec2::new(ui.available_width(), 30.0), Sense::click());
		let hovered = response.hovered();
		let fill = if self.selected {
			Color32::from_rgb(20, 92, 118)
		} else if hovered {
			Color32::from_rgb(25, 34, 48)
		} else if self.index % 2 == 0 {
			Color32::from_rgb(15, 20, 28)
		} else {
			Color32::from_rgb(13, 18, 25)
		};
		let stroke = if self.selected {
			Stroke::new(1.0, palette::ACCENT)
		} else {
			Stroke::new(1.0, Color32::TRANSPARENT)
		};

		ui.painter().rect(
			rect.shrink2(Vec2::new(0.0, 1.0)),
			6.0,
			fill,
			stroke,
			egui::StrokeKind::Inside,
		);
		ui.painter().text(
			egui::pos2(rect.left() + 10.0, rect.center().y),
			egui::Align2::LEFT_CENTER,
			format!("Lazy row {}", self.index),
			FontId::proportional(13.0),
			if self.selected { Color32::WHITE } else { palette::TEXT },
		);
		ui.painter().text(
			egui::pos2(rect.right() - 10.0, rect.center().y),
			egui::Align2::RIGHT_CENTER,
			format!("#{:04}", self.index),
			FontId::monospace(12.0),
			if self.selected {
				Color32::from_rgb(206, 244, 255)
			} else {
				palette::MUTED
			},
		);

		response
	}
}

fn active_meter_color(step_ratio: f32) -> Color32 {
	if step_ratio < 0.5 {
		let t = step_ratio / 0.5;
		Color32::from_rgb((34.0 + 120.0 * t) as u8, 211, (238.0 - 120.0 * t) as u8)
	} else {
		let t = (step_ratio - 0.5) / 0.5;
		Color32::from_rgb(
			(palette::ACCENT_GREEN.r() as f32 + (palette::ACCENT_ORANGE.r() as f32 - palette::ACCENT_GREEN.r() as f32) * t)
				as u8,
			(palette::ACCENT_GREEN.g() as f32 + (palette::ACCENT_ORANGE.g() as f32 - palette::ACCENT_GREEN.g() as f32) * t)
				as u8,
			(palette::ACCENT_GREEN.b() as f32 + (palette::ACCENT_ORANGE.b() as f32 - palette::ACCENT_GREEN.b() as f32) * t)
				as u8,
		)
	}
}
