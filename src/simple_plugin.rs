use std::{collections::VecDeque, num::NonZeroU32, sync::Arc, time::Instant};

use crate::widgets::{self as ui_kit, palette};
use nih_plug::prelude::*;
use nih_plug_egui::{
	create_egui_editor,
	egui::{self, RichText, Stroke, UiBuilder, Vec2},
	resizable_window::ResizableWindow,
	EguiState,
};

const GAIN_STEPS: usize = 24;
const LAZY_ITEM_COUNT: usize = 2_000;
const OUTER_PADDING: f32 = 14.0;

#[derive(Default)]
struct DiagnosticState {
	text_value: String,
	clipboard_status: String,
	selected_lazy_item: Option<usize>,
	lazy_visible_range: (usize, usize),
	hover_count: usize,
	shortcut_count: usize,
	last_key: String,
	last_mouse: String,
	frame_last_at: Option<Instant>,
	frame_intervals_ms: VecDeque<f32>,
	frame_count: u64,
	frame_rate_text: String,
}

impl DiagnosticState {
	fn new() -> Self {
		Self {
			text_value: "Type here. Try IME, option characters, paste, backspace.".to_string(),
			clipboard_status: "Clipboard not tested".to_string(),
			last_key: "No key yet".to_string(),
			last_mouse: "Move over hover pads".to_string(),
			frame_intervals_ms: VecDeque::with_capacity(120),
			frame_rate_text: "collecting".to_string(),
			..Self::default()
		}
	}

	fn record_frame(&mut self) {
		let now = Instant::now();
		self.frame_count = self.frame_count.saturating_add(1);

		if let Some(previous) = self.frame_last_at.replace(now) {
			let ms = now.duration_since(previous).as_secs_f32() * 1000.0;
			self.frame_intervals_ms.push_back(ms);
			while self.frame_intervals_ms.len() > 90 {
				self.frame_intervals_ms.pop_front();
			}

			let avg_ms = self.frame_intervals_ms.iter().sum::<f32>() / self.frame_intervals_ms.len().max(1) as f32;
			let fps = if avg_ms > 0.0 { 1000.0 / avg_ms } else { 0.0 };
			self.frame_rate_text = format!("{fps:.1} fps avg {avg_ms:.1} ms");
		}
	}
}

#[derive(Params)]
pub struct SimpleParams {
	#[id = "gain"]
	pub gain: FloatParam,

	#[persist = "egui_state"]
	egui_state: Arc<EguiState>,
}

pub struct SimplePlugin {
	params: Arc<SimpleParams>,
}

impl Default for SimplePlugin {
	fn default() -> Self {
		Self {
			params: Arc::new(SimpleParams {
				gain: FloatParam::new("Gain", 1.0, FloatRange::Linear { min: 0.0, max: 1.0 })
					.with_smoother(SmoothingStyle::Linear(20.0))
					.with_unit("%")
					.with_value_to_string(Arc::new(|value| format!("{:.0}", value * 100.0))),
				egui_state: EguiState::from_size(980, 680),
			}),
		}
	}
}

impl Plugin for SimplePlugin {
	const NAME: &'static str = "simple-egui-plugin";
	const VENDOR: &'static str = "nih-plug-egui-test";
	const URL: &'static str = env!("CARGO_PKG_HOMEPAGE");
	const EMAIL: &'static str = "";
	const VERSION: &'static str = env!("CARGO_PKG_VERSION");

	const AUDIO_IO_LAYOUTS: &'static [AudioIOLayout] = &[AudioIOLayout {
		main_input_channels: NonZeroU32::new(2),
		main_output_channels: NonZeroU32::new(2),
		..AudioIOLayout::const_default()
	}];

	const MIDI_INPUT: MidiConfig = MidiConfig::Basic;
	const SAMPLE_ACCURATE_AUTOMATION: bool = true;

	type SysExMessage = ();
	type BackgroundTask = ();

	fn params(&self) -> Arc<dyn Params> {
		self.params.clone()
	}

	fn process(
		&mut self,
		buffer: &mut Buffer,
		_: &mut AuxiliaryBuffers,
		_: &mut impl ProcessContext<Self>,
	) -> ProcessStatus {
		for channel_samples in buffer.iter_samples() {
			let gain = self.params.gain.smoothed.next();

			for sample in channel_samples {
				*sample *= gain;
			}
		}

		ProcessStatus::Normal
	}

	fn editor(&mut self, _async_executor: AsyncExecutor<Self>) -> Option<Box<dyn Editor>> {
		let params = self.params.clone();
		let egui_state = params.egui_state.clone();

		create_egui_editor(
			egui_state.clone(),
			DiagnosticState::new(),
			|_, _| {},
			move |egui_ctx, setter, state| {
				ui_kit::apply_theme(egui_ctx);
				egui_ctx.request_repaint();
				state.record_frame();
				handle_shortcuts(egui_ctx, setter, state, &params);

				egui::CentralPanel::default()
					.frame(egui::Frame::default().fill(palette::APP_BG))
					.show(egui_ctx, |ui| {
						ResizableWindow::new("simple-egui-plugin-window")
							.min_size(Vec2::new(760.0, 520.0))
							.handle_margin(OUTER_PADDING)
							.show(ui.ctx(), egui_state.as_ref(), |ui| {
								let container_rect = ui.max_rect().shrink(0.5);
								ui.painter().rect(
									container_rect,
									10.0,
									palette::PANEL_BG,
									Stroke::new(1.0, palette::CARD_STROKE),
									egui::StrokeKind::Inside,
								);

								let inner_rect = container_rect.shrink(18.0);
								let mut content_ui = ui.new_child(UiBuilder::new().max_rect(inner_rect).layout(*ui.layout()));
								content_ui.spacing_mut().item_spacing = Vec2::new(14.0, 14.0);
								draw_header(&mut content_ui, state, &params);
								content_ui.add_space(2.0);

								egui::ScrollArea::vertical()
									.auto_shrink([false, false])
									.show(&mut content_ui, |ui| {
										ui.columns(3, |columns| {
											columns[0].vertical(|ui| {
												host_and_parameter_sync(ui, setter, &params);
												animation_probe(ui, state, params.gain.value());
												hover_and_cursors(ui, state);
											});

											columns[1].vertical(|ui| {
												text_and_keyboard_probe(ui, state);
												resize_scale_focus_probe(ui, state, egui_state.as_ref());
												tooltip_probe(ui);
											});

											columns[2].vertical(|ui| {
												lazy_list_probe(ui, state);
												manual_matrix(ui);
											});
										});
									});
							});
					});
			},
		)
	}
}

impl ClapPlugin for SimplePlugin {
	const CLAP_ID: &'static str = "com.nih-plug-egui-test.simple";
	const CLAP_DESCRIPTION: Option<&'static str> = Some("A simple nih-plug egui editor plugin");
	const CLAP_MANUAL_URL: Option<&'static str> = Some(Self::URL);
	const CLAP_SUPPORT_URL: Option<&'static str> = None;
	const CLAP_FEATURES: &'static [ClapFeature] = &[ClapFeature::AudioEffect, ClapFeature::Stereo];
}

impl Vst3Plugin for SimplePlugin {
	const VST3_CLASS_ID: [u8; 16] = *b"SimpleEguiPlugin";
	const VST3_SUBCATEGORIES: &'static [Vst3SubCategory] = &[Vst3SubCategory::Fx];
}

fn draw_header(ui: &mut egui::Ui, state: &DiagnosticState, params: &SimpleParams) {
	egui::Frame::default()
		.fill(palette::PANEL_BG)
		.stroke(Stroke::new(1.0, palette::CARD_STROKE))
		.corner_radius(8)
		.inner_margin(egui::Margin::symmetric(16, 14))
		.show(ui, |ui| {
			ui.horizontal(|ui| {
				ui.vertical(|ui| {
					ui.label(
						RichText::new("egui embed diagnostics")
							.size(24.0)
							.strong()
							.color(palette::TEXT),
					);
					ui.add_space(3.0);
					ui.label(
						RichText::new("Host integration probes for parameters, input, rendering, and window lifecycle")
							.size(12.0)
							.color(palette::MUTED),
					);
				});

				ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
					ui.add(ui_kit::Pill::new(
						"repaint",
						state.frame_rate_text.clone(),
						palette::ACCENT_GREEN,
					));
					ui.add(ui_kit::Pill::new("gain", params.gain.to_string(), palette::ACCENT));
				});
			});
		});
}

fn handle_shortcuts(
	egui_ctx: &egui::Context,
	setter: &ParamSetter,
	state: &mut DiagnosticState,
	params: &SimpleParams,
) {
	let shortcut = egui_ctx.input(|input| {
		let command = input.modifiers.command;
		if command && input.key_pressed(egui::Key::G) {
			Some(-0.05)
		} else if command && input.key_pressed(egui::Key::H) {
			Some(0.05)
		} else {
			None
		}
	});

	if let Some(delta) = shortcut {
		state.shortcut_count += 1;
		let normalized = (params.gain.value() + delta).clamp(0.0, 1.0);
		set_gain_normalized(setter, params, normalized);
	}

	if let Some(key) = egui_ctx.input(|input| {
		input.events.iter().rev().find_map(|event| match event {
			egui::Event::Key { key, pressed: true, .. } => Some(format!("{key:?}")),
			egui::Event::Text(text) => Some(format!("text {text:?}")),
			_ => None,
		})
	}) {
		state.last_key = key;
	}
}

fn set_gain_normalized(setter: &ParamSetter, params: &SimpleParams, normalized: f32) {
	setter.begin_set_parameter(&params.gain);
	setter.set_parameter_normalized(&params.gain, normalized.clamp(0.0, 1.0));
	setter.end_set_parameter(&params.gain);
}

fn section(ui: &mut egui::Ui, title: &str, body: impl FnOnce(&mut egui::Ui)) {
	ui_kit::Card::new(title).show(ui, body);
}

fn status_chip(ui: &mut egui::Ui, label: &str, value: impl Into<String>) {
	ui.add(ui_kit::StatusRow::new(label, value));
}

fn action_button(ui: &mut egui::Ui, label: &str) -> egui::Response {
	ui.add(ui_kit::ActionButton::new(label))
}

fn host_and_parameter_sync(ui: &mut egui::Ui, setter: &ParamSetter, params: &SimpleParams) {
	section(ui, "Host and parameter sync", |ui| {
		let mut normalized = params.gain.value().clamp(0.0, 1.0);

		ui.horizontal(|ui| {
			ui.vertical(|ui| {
				ui.label(RichText::new("GAIN").size(11.0).color(palette::SUBTLE).strong());
				ui.label(RichText::new(params.gain.to_string()).size(28.0).color(palette::ACCENT));
			});
			ui.with_layout(egui::Layout::right_to_left(egui::Align::BOTTOM), |ui| {
				ui.label(
					RichText::new(format!("host {:.0}%", normalized * 100.0))
						.size(12.0)
						.color(palette::ACCENT_GREEN)
						.strong(),
				);
			});
		});

		let mut slider_value = normalized;
		if ui.add(ui_kit::NormalizedSlider::new(&mut slider_value)).changed() {
			set_gain_normalized(setter, params, slider_value);
			normalized = slider_value;
		}
		let mut meter_value = normalized;
		if ui
			.add(ui_kit::SegmentedMeter::new(&mut meter_value).steps(GAIN_STEPS))
			.changed()
		{
			set_gain_normalized(setter, params, meter_value);
		}

		ui.horizontal(|ui| {
			if action_button(ui, "-5%").clicked() {
				set_gain_normalized(setter, params, normalized - 0.05);
			}
			if action_button(ui, "Reset").clicked() {
				set_gain_normalized(setter, params, params.gain.default_normalized_value());
			}
			if action_button(ui, "+5%").clicked() {
				set_gain_normalized(setter, params, normalized + 0.05);
			}
		});
	});
}

fn animation_probe(ui: &mut egui::Ui, state: &DiagnosticState, gain: f32) {
	section(ui, "Animation and presentation", |ui| {
		ui.add(ui_kit::AnimationBars::new(state.frame_count, gain));
		status_chip(ui, "Expected", "bars repaint continuously");
		status_chip(ui, "Frame probe", state.frame_rate_text.clone());
	});
}

fn hover_and_cursors(ui: &mut egui::Ui, state: &mut DiagnosticState) {
	section(ui, "Hover and cursors", |ui| {
		let pad_width = ((ui.available_width() - 8.0) / 2.0).max(96.0);
		ui.horizontal(|ui| {
			hover_pad(ui, state, "pointer", egui::CursorIcon::PointingHand, pad_width);
			hover_pad(ui, state, "text", egui::CursorIcon::Text, pad_width);
		});
		ui.horizontal(|ui| {
			hover_pad(ui, state, "crosshair", egui::CursorIcon::Crosshair, pad_width);
			hover_pad(ui, state, "ew-resize", egui::CursorIcon::ResizeHorizontal, pad_width);
		});
		status_chip(ui, "Mouse", state.last_mouse.clone());
		status_chip(ui, "Hover moves", state.hover_count.to_string());
	});
}

fn hover_pad(
	ui: &mut egui::Ui,
	state: &mut DiagnosticState,
	label: &'static str,
	cursor: egui::CursorIcon,
	width: f32,
) {
	let response = ui.add(ui_kit::HoverPad::new(label, cursor, width));
	if response.hovered() {
		state.hover_count = state.hover_count.saturating_add(1);
		if let Some(position) = response.hover_pos() {
			state.last_mouse = format!("x={:.0} y={:.0}", position.x, position.y);
		}
	}
}

fn text_and_keyboard_probe(ui: &mut egui::Ui, state: &mut DiagnosticState) {
	section(ui, "Focus, keyboard, text, IME", |ui| {
		ui.add(
			egui::TextEdit::multiline(&mut state.text_value)
				.desired_rows(4)
				.desired_width(ui.available_width()),
		);

		ui.horizontal(|ui| {
			if action_button(ui, "Copy").clicked() {
				ui.ctx().copy_text(state.text_value.clone());
				state.clipboard_status = format!("Copied {} chars", state.text_value.chars().count());
			}
			if action_button(ui, "Clear").clicked() {
				state.text_value.clear();
				state.clipboard_status = "Cleared text".to_string();
			}
		});

		status_chip(ui, "Last key", state.last_key.clone());
		status_chip(ui, "Shortcut", format!("cmd-g/cmd-h count {}", state.shortcut_count));
		status_chip(ui, "Clipboard", state.clipboard_status.clone());
	});
}

fn resize_scale_focus_probe(ui: &mut egui::Ui, state: &DiagnosticState, egui_state: &EguiState) {
	section(ui, "Resize, scale, close, accessibility", |ui| {
		let (width, height) = egui_state.size();
		status_chip(ui, "Window logical", format!("{width} x {height}"));
		status_chip(ui, "Scale factor", format!("{:.2}", ui.ctx().pixels_per_point()));
		status_chip(
			ui,
			"Any focus",
			ui.memory(|memory| memory.focused().is_some()).to_string(),
		);
		status_chip(ui, "Frame count", state.frame_count.to_string());
	});
}

fn tooltip_probe(ui: &mut egui::Ui) {
	section(ui, "Tooltip and popup positioning", |ui| {
		ui.add(
			ui_kit::ActionButton::new("Hover for tooltip")
				.min_size(Vec2::new(ui.available_width(), 46.0))
				.corner_radius(8),
		)
		.on_hover_text("Tooltip rendered inside embedded egui");
		status_chip(ui, "Expected", "tooltip near target, no host offset bug");
	});
}

fn lazy_list_probe(ui: &mut egui::Ui, state: &mut DiagnosticState) {
	section(ui, "Lazy dynamic list", |ui| {
		let row_height = 34.0;
		let mut visible = (usize::MAX, 0);
		egui::ScrollArea::vertical()
			.max_height(360.0)
			.auto_shrink([false, false])
			.show_rows(ui, row_height, LAZY_ITEM_COUNT, |ui, range| {
				visible = (range.start, range.end);
				for ix in range {
					let response = ui.add(ui_kit::LazyRow::new(ix, state.selected_lazy_item == Some(ix)));
					if response.clicked() {
						state.selected_lazy_item = Some(ix);
					}
				}
			});

		if visible.0 != usize::MAX {
			state.lazy_visible_range = visible;
		}

		ui.horizontal(|ui| {
			if action_button(ui, "Top").clicked() {
				state.selected_lazy_item = Some(0);
			}
			if action_button(ui, "Middle").clicked() {
				state.selected_lazy_item = Some(LAZY_ITEM_COUNT / 2);
			}
			if action_button(ui, "End").clicked() {
				state.selected_lazy_item = Some(LAZY_ITEM_COUNT - 1);
			}
		});
		status_chip(
			ui,
			"Visible rows",
			format!("{}..{}", state.lazy_visible_range.0, state.lazy_visible_range.1),
		);
		status_chip(
			ui,
			"Selected",
			state
				.selected_lazy_item
				.map(|ix| ix.to_string())
				.unwrap_or_else(|| "none".to_string()),
		);
	});
}

fn manual_matrix(ui: &mut egui::Ui) {
	section(ui, "Manual matrix", |ui| {
		status_chip(ui, "DAW param", "drag gain in host");
		status_chip(ui, "Multi-instance", "open 2 editors");
		status_chip(ui, "Teardown", "close/rescan/unload");
		status_chip(ui, "HiDPI", "move across displays");
	});
}
