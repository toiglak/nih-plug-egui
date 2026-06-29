use std::{num::NonZeroU32, sync::Arc};

use nih_plug::prelude::*;
use nih_plug_egui::{
	create_egui_editor,
	egui::{self, Vec2},
	resizable_window::ResizableWindow,
	widgets, EguiState,
};

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
					.with_unit("%")
					.with_value_to_string(Arc::new(|value| format!("{:.0}", value * 100.0))),
				egui_state: EguiState::from_size(420, 220),
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
			(),
			|_, _| {},
			move |egui_ctx, setter, _state| {
				egui::CentralPanel::default()
					.frame(egui::Frame::default().inner_margin(egui::Margin::same(16)))
					.show(egui_ctx, |ui| {
						ResizableWindow::new("simple-egui-plugin-window")
							.min_size(Vec2::new(320.0, 180.0))
							.show(ui.ctx(), egui_state.as_ref(), |ui| {
								ui.heading("simple-egui-plugin");
								ui.label("A minimal nih-plug-egui editor.");
								ui.add_space(12.0);

								ui.label("Gain");
								ui.add(widgets::ParamSlider::for_param(&params.gain, setter));

								ui.add_space(8.0);
								ui.label(format!("Current value: {}", params.gain.to_string()));
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
